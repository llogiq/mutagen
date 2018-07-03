use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::fold::{self, Folder};
use syntax::ptr::P;
use syntax::symbol::Symbol;
use mutagen::bounded_loop::LoopId;
use syntax::util::small_vector::SmallVector;
use std::mem;
use super::Resizer;

static LOOP_COUNT: AtomicUsize = AtomicUsize::new(1);

/// expand all loops found on annotated trait or impl. When executed with no mutations (so,
/// mutation_count == 0), it will count the maximum bound for each of the loops.
/// When executed with mutations, we will load the bound for each loop and, if some loop
/// reaches that bound, we will end the current process.
pub fn bounded_loop(cx: &mut ExtCtxt, _span: Span, _mi: &MetaItem, a: Annotatable) -> Annotatable {
    let current_id = LOOP_COUNT.load(SeqCst);
    let mut plugin = Plugin::new(cx, LoopId::new(current_id));
    LOOP_COUNT.store(plugin.loop_count.id(), SeqCst);

    match a {
        Annotatable::Item(i) => {
            Annotatable::Item(plugin.fold_item(i).expect_one("expected exactly one item"))
        }
        Annotatable::TraitItem(i) => Annotatable::TraitItem(i.map(|i| {
            plugin.fold_trait_item(i).expect_one("expected exactly one item")
        })),
        Annotatable::ImplItem(i) => Annotatable::ImplItem(i.map(|i| {
            plugin.fold_impl_item(i).expect_one("expected exactly one item")
        })),
        a => a,
    }
}

struct Plugin<'a, 'cx: 'a> {
    cx: &'a mut ExtCtxt<'cx>,

    /// Loop id for the following loop that will be mutated.
    loop_count: LoopId,

    /// Loop id of the first loop on a function or method. This is updated when a new function or method
    /// is mutated.
    block_first: LoopId,
}

impl<'a, 'cx: 'a> Plugin<'a, 'cx> {
    pub fn new(cx: &'a mut ExtCtxt<'cx>, loop_count: LoopId) -> Self {
        Plugin {
            cx,
            loop_count,
            block_first: LoopId::new(0usize),
        }
    }

    fn method(&mut self, block: P<Block>) -> P<Block> {
        let first = self.loop_count.id();
        self.block_first = self.loop_count.clone();

        let pre_stmts = vec![
            quote_stmt!(self.cx, static __LOOP_COUNTERS : [::std::sync::atomic::AtomicUsize; 0] = [::std::sync::atomic::ATOMIC_USIZE_INIT; 0];).unwrap(),
        ];

        block.map(
            |Block {
                stmts,
                id,
                rules,
                span,
                recovered,
            } | {
                let mut newstmts: Vec<Stmt> = Vec::with_capacity(pre_stmts.len() + stmts.len());
                newstmts.extend(pre_stmts);
                newstmts.extend(stmts.into_iter().flat_map(|s| fold::noop_fold_stmt(s, self)));

                let mut resizer = Resizer(self.loop_count.id() - first);
                let counters = mem::replace(&mut newstmts[0], quote_stmt!(self.cx, ();).unwrap());
                let _ = mem::replace(&mut newstmts[0], resizer.fold_stmt(counters).expect_one("?"));

                Block {
                    stmts: newstmts,
                    id,
                    rules,
                    span,
                    recovered,
                }
            }
        )
    }

    fn wrap_block(&mut self, loop_id: LoopId, block: P<Block>) -> (P<Block>, Ident) {
        let loop_id = loop_id.id();
        // TODO: Test 2-level loop
        let block = self.fold_block(block);
        let sym = Symbol::gensym(&format!("__mutagen_loop_id{}", loop_id));
        let ident = Ident::with_empty_ctxt(sym);

        let block = quote_block!(self.cx, {
                    $ident.step();

                    $block
                });

        (block, ident)
    }

    fn wrap_expression(&mut self, expr: P<Expr>, current_id: LoopId, symbol: Ident) -> P<Expr> {
        let loop_id = current_id.id();
        let first = self.block_first.id();

        quote_expr!(self.cx, {
                    use ::mutagen::bounded_loop::LoopStep;
                    use ::mutagen::bounded_loop::LoopCount;
                    use ::mutagen::bounded_loop::LoopId;
                    use ::mutagen::bounded_loop::LoopBound;

                    if ::mutagen::get() == 0usize {
                        let mut $symbol = LoopCount::new(LoopId::new($loop_id), &__LOOP_COUNTERS[$loop_id - $first]);

                        $expr
                    } else {
                        let mut $symbol = LoopBound::new(LoopId::new($loop_id));

                        $expr
                    }
                })
    }
}

impl<'a, 'cx: 'a> Folder for Plugin<'a, 'cx> {
    fn fold_impl_item(&mut self, i: ImplItem) -> SmallVector<ImplItem> {
        SmallVector::one(match i {
            ImplItem {
                id,
                ident,
                vis,
                defaultness,
                attrs,
                generics,
                node: ImplItemKind::Method(sig, block),
                span,
                tokens,
            } => {
                let ii = ImplItem {
                    id,
                    ident,
                    vis,
                    defaultness,
                    attrs,
                    generics,
                    node: ImplItemKind::Method(sig, self.method(block)),
                    span,
                    tokens,
                };

                ii
            },
            ii => ii,
        })
    }

    fn fold_trait_item(&mut self, i: TraitItem) -> SmallVector<TraitItem> {
        SmallVector::one(match i {
            TraitItem {
                id,
                ident,
                attrs,
                generics,
                node: TraitItemKind::Method(sig, Some(block)),
                span,
                tokens,
            } => {
                let ti = TraitItem {
                    id,
                    ident,
                    attrs,
                    generics,
                    node: TraitItemKind::Method(sig, Some(self.method(block))),
                    span,
                    tokens,
                };

                ti
            }
            ti => ti,
        })
    }

    fn fold_item_kind(&mut self, i: ItemKind) -> ItemKind {
        match i {
            ItemKind::Fn(decl, header, generics, block) => {
                let k = ItemKind::Fn(
                    decl,
                    header,
                    generics,
                    self.method(block),
                );
                k
            }
            s @ ItemKind::Static(..) | s @ ItemKind::Const(..) => s,
            k => fold::noop_fold_item_kind(k, self),
        }
    }

    fn fold_expr(&mut self, expr: P<Expr>) -> P<Expr> {
        expr.and_then(|expr| match expr {
            e @ Expr {
                id: _,
                node: ExprKind::Mac(_),
                span: _,
                attrs: _,
            } => {
                // ignore macros for now
                P(e)
            }
            Expr {
                id,
                node: ExprKind::Loop(block, opt_label),
                span,
                attrs,
            } => {
                let current = self.loop_count;
                self.loop_count = self.loop_count.next();
                let (block, sym) = self.wrap_block(current, block);

                let e = P(Expr {
                    id,
                    node: ExprKind::Loop(block, opt_label),
                    span,
                    attrs
                });

                self.wrap_expression(e, current, sym)

            }
            Expr {
                id,
                node: ExprKind::While(expr, block, opt_span),
                span,
                attrs,
            } => {
                let current = self.loop_count;
                self.loop_count = self.loop_count.next();
                let (block, sym) = self.wrap_block(current, block);

                let e = P(Expr {
                    id,
                    node: ExprKind::While(expr, block, opt_span),
                    span,
                    attrs
                });

                self.wrap_expression(e, current, sym)

            }
            Expr {
                id,
                node: ExprKind::WhileLet(pat, expr, block, opt_span),
                span,
                attrs,
            } => {
                let current = self.loop_count;
                self.loop_count = self.loop_count.next();
                let (block, sym) = self.wrap_block(current, block);

                let e = P(Expr {
                    id,
                    node: ExprKind::WhileLet(pat, expr, block, opt_span),
                    span,
                    attrs
                });

                self.wrap_expression(e, current, sym)

            }
            Expr {
                id,
                node: ExprKind::ForLoop(pat, expr, block, opt_span),
                span,
                attrs,
            } => {
                let current = self.loop_count;
                self.loop_count = self.loop_count.next();
                let (block, sym) = self.wrap_block(current, block);

                let e = P(Expr {
                    id,
                    node: ExprKind::ForLoop(pat, expr, block, opt_span),
                    span,
                    attrs
                });

                self.wrap_expression(e, current, sym)

            }
            e => P(fold::noop_fold_expr(e, self)),
        })
    }

    fn fold_mac(&mut self, mac: Mac) -> Mac {
        mac
    }
}
