use crate::general::TakePutBack;
use nonempty::NonEmpty;

impl<M: Default> TakePutBack<usize, usize> for Vec<M> {
    type ItemType = M;
    type PutType = M;

    fn take(&mut self, index_into: usize) -> Self::ItemType {
        std::mem::take(&mut self[index_into])
    }

    fn put_back(&mut self, index_into: usize, reinsert: Self::PutType) {
        self[index_into] = reinsert;
    }
}

impl<M: Default> TakePutBack<usize, usize> for NonEmpty<M> {
    type ItemType = M;
    type PutType = M;

    fn take(&mut self, index_into: usize) -> Self::ItemType {
        if index_into == 0 {
            std::mem::take(&mut self.head)
        } else {
            self.tail.take(index_into - 1)
        }
    }

    fn put_back(&mut self, index_into: usize, reinsert: Self::PutType) {
        self[index_into] = reinsert;
    }
}

mod test {

    #[test]
    fn vec_i32() {
        use super::TakePutBack;
        let mut v = vec![0, 1, 2, 3, 4, 5];
        let expected = vec![0, 2, 2, 4, 4, 6];
        v.process_all::<fn(i32) -> i32>(&[(1, 1), (3, 3), (5, 5)], |x| x + 1);
        assert_eq!(v, expected);
    }

    #[test]
    fn nonempty_i32() {
        use super::TakePutBack;
        let mut v = nonempty::nonempty![0, 1, 2, 3, 4, 5];
        let expected = nonempty::nonempty![1, 2, 2, 4, 4, 6];
        v.process_all::<fn(i32) -> i32>(&[(0, 0), (1, 1), (3, 3), (5, 5)], |x| x + 1);
        assert_eq!(v, expected);
    }
}
