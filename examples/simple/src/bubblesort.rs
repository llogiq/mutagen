use mutagen::mutate;

#[cfg_attr(test, mutate)]
pub fn bubblesort_for(arr: &mut [u8]) {
    let n = arr.len();
    for _ in 1..n {
        for i in 1..n {
            if arr[i - 1] > arr[i] {
                arr.swap(i - 1, i);
            }
        }
    }
}

#[cfg_attr(test, mutate)]
pub fn bubblesort_while(arr: &mut [u8]) {
    let n = arr.len();
    let mut change = true;
    while change {
        change = false;
        for i in 1..n {
            if arr[i - 1] > arr[i] {
                arr.swap(i - 1, i);
                change = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_bubblesort_for_123() {
        let mut arr = vec![1, 2, 3];
        bubblesort_for(&mut arr);
        assert_eq!(&*arr, [1, 2, 3]);
    }
    #[test]
    fn test_bubblesort_for_321() {
        let mut arr = vec![3, 2, 1];
        bubblesort_for(&mut arr);
        assert_eq!(&*arr, [1, 2, 3]);
    }

    #[test]
    fn test_bubblesort_while_123() {
        let mut arr = vec![1, 2, 3];
        bubblesort_while(&mut arr);
        assert_eq!(&*arr, [1, 2, 3]);
    }
    #[test]
    fn test_bubblesort_while_321() {
        let mut arr = vec![3, 2, 1];
        bubblesort_while(&mut arr);
        assert_eq!(&*arr, [1, 2, 3]);
    }
}
