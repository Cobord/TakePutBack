use std::{num::NonZeroUsize, sync::mpsc, thread};

#[allow(dead_code)]
pub trait TakePutBack<IndexInto1: Clone, IndexInto2: Clone + Send + 'static> {
    type ItemType;
    type PutType;

    /// you can extract out the `ItemType` at `index_into`
    /// leaving a default in it's place
    /// or something else that is up to the implementer
    fn take(&mut self, index_into: IndexInto1) -> Self::ItemType;

    /// splice in `PutType` at the prescibed location
    /// the way you index for taking and putting back do not have to be the same
    fn put_back(&mut self, index_into: IndexInto2, reinsert: Self::PutType);

    /// the way you index for taking and putting back do not have to be the same
    /// but for the do nothing operation, there is still some correspondence
    /// as described below
    fn all_idces_inout(&self) -> Vec<(IndexInto1, IndexInto2)>;

    /// if you do take on `index_into` and get some `ItemType`,
    /// run that through this function and then `put_back`
    /// with the corrsponding `index_into` (the one in the same pair in `all_idces_inout`)
    /// the composite operation should be the identity
    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType;

    /// for each pair in `which_idces`
    /// we are taking out with the first component
    /// applying the `processor`
    /// then doing `put_back` with the second component
    /// all the entries in `which_idces` should be independent
    /// it is assumed that the `processor` running is the intense
    /// part so those are all operating with their own `thread::spawn`
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

    /// as before but this has only at most the `thread::available_parallism()`
    /// in `which_idces`, could call this directly but that avoids the chunking
    /// into such bounded sizes that `process_all` does
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
        for (idx1, idx2) in which_idces {
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
