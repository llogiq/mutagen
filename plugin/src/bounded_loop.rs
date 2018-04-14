use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use syntax::ast::*;
use syntax::codemap::Span;
use syntax::ext::base::{Annotatable, ExtCtxt};
use syntax::fold::{self, Folder};
use syntax::ptr::P;
use syntax::symbol::Symbol;
use mutagen::LoopId;

static LOOP_COUNT: AtomicUsize = AtomicUsize::new(1);

/// expand all loops found on annotated trait or impl. When there will be no mutations (the code
/// is executed with mutation_count == 0), it will count the maximum bound for that loops.
/// When we will execute with mutations, we will load the bound for each loop and, if some loop
/// reaches an imposed bound, we will end the current process.
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
    loop_count: LoopId,
}

impl<'a, 'cx: 'a> Plugin<'a, 'cx> {
    pub fn new(cx: &'a mut ExtCtxt<'cx>, loop_count: LoopId) -> Self {
        Plugin {
            cx,
            loop_count,
        }
    }
}

impl<'a, 'cx: 'a> Folder for Plugin<'a, 'cx> {
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
                self.loop_count = self.loop_count.next();
                let loop_id = self.loop_count.id();
                let sym = Symbol::gensym(&format!("__mutagen_loop_id{}", loop_id));
                let s = sym.to_ident();
                let block = self.fold_block(block);
                let block = quote_block!(self.cx, {
                    $s.step();

                    $block
                });

                let e = P(Expr {
                    id,
                    node: ExprKind::Loop(block, opt_label),
                    span,
                    attrs
                });

                quote_expr!(self.cx, {
                    let mut $s = if ::mutagen::get() == 0usize {
                        ::mutagen::LoopCounter::recording(::mutagen::LoopId::new($loop_id))
                    } else {
                        ::mutagen::LoopCounter::bounded(::mutagen::LoopId::new($loop_id))
                    };

                    $e
                })
            }
            e => P(fold::noop_fold_expr(e, self)),
        })
    }

    fn fold_mac(&mut self, mac: Mac) -> Mac {
        mac
    }
}