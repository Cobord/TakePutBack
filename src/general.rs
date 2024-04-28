use std::{num::NonZeroUsize, sync::mpsc, thread};

pub trait TakePutBack<IndexInto1: Clone, IndexInto2: Clone + Send + 'static> {
    type ItemType;
    type PutType;

    fn take(&mut self, index_into: IndexInto1) -> Self::ItemType;

    fn put_back(&mut self, index_into: IndexInto2, reinsert: Self::PutType);

    fn all_idces_inout(&self) -> Vec<(IndexInto1, IndexInto2)>;
    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType;

    fn process_all<F>(
        &mut self,
        which_idces: &[(IndexInto1, IndexInto2)],
        processor: fn(Self::ItemType) -> Self::PutType,
    ) where
        Self::PutType: Send + 'static,
        Self::ItemType: Send + 'static,
    {
        let max_threads = thread::available_parallelism().unwrap_or(NonZeroUsize::new(4).unwrap());
        let chunks = which_idces.chunks(max_threads.into());
        for chunk in chunks {
            self.process_all_helper::<F>(chunk, processor);
        }
    }

    fn process_all_helper<F>(
        &mut self,
        which_idces: &[(IndexInto1, IndexInto2)],
        processor: fn(Self::ItemType) -> Self::PutType,
    ) where
        Self::PutType: Send + 'static,
        Self::ItemType: Send + 'static,
    {
        let mut jh = Vec::with_capacity(which_idces.len());
        let (tx, rx) = mpsc::channel();
        for (idx1, idx2) in which_idces.iter() {
            let cur_item = self.take(idx1.clone());
            let put_this_back_here = idx2.clone();
            let my_sender = tx.clone();
            jh.push(thread::spawn(move || {
                let to_put_back = processor(cur_item);
                my_sender
                    .send((put_this_back_here, to_put_back))
                    .expect("Problem sending");
            }));
        }
        drop(tx);
        loop {
            let b = rx.recv();
            if let Ok((index_into, reinsert)) = b {
                self.put_back(index_into, reinsert);
            } else {
                break;
            }
        }
    }
}
