use log::info;
use nonempty::NonEmpty;
use petgraph::{
    prelude::{EdgeIndex, NodeIndex},
    stable_graph::StableGraph,
    visit::EdgeRef,
    Direction, Graph,
};
use std::{num::NonZeroUsize, sync::mpsc, thread};

pub trait TakePutBack<IndexInto1: Clone, IndexInto2: Clone + Send + 'static> {
    type ItemType;
    type PutType;

    fn take(&mut self, index_into: IndexInto1) -> Self::ItemType;

    fn put_back(&mut self, index_into: IndexInto2, reinsert: Self::PutType);

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

impl<N: Default, E> TakePutBack<NodeIndex, NodeIndex> for StableGraph<N, E> {
    type ItemType = N;
    type PutType = N;

    fn take(&mut self, index_into: NodeIndex) -> Self::ItemType {
        let z = self.node_weight_mut(index_into).expect("Index Exists");
        std::mem::take(z)
    }

    fn put_back(&mut self, index_into: NodeIndex, reinsert: Self::PutType) {
        let z = self.node_weight_mut(index_into).expect("Index Exists");
        *z = reinsert;
    }
}

impl<N: Default, E> TakePutBack<NodeIndex, NodeIndex> for Graph<N, E> {
    type ItemType = N;
    type PutType = N;

    fn take(&mut self, index_into: NodeIndex) -> Self::ItemType {
        let z = self.node_weight_mut(index_into).expect("Index Exists");
        std::mem::take(z)
    }

    fn put_back(&mut self, index_into: NodeIndex, reinsert: Self::PutType) {
        let z = self.node_weight_mut(index_into).expect("Index Exists");
        *z = reinsert;
    }
}

impl<N, E: Default> TakePutBack<EdgeIndex, EdgeIndex> for StableGraph<N, E> {
    type ItemType = E;
    type PutType = E;

    fn take(&mut self, index_into: EdgeIndex) -> Self::ItemType {
        let z = self.edge_weight_mut(index_into).expect("Index Exists");
        std::mem::take(z)
    }

    fn put_back(&mut self, index_into: EdgeIndex, reinsert: Self::PutType) {
        let z = self.edge_weight_mut(index_into).expect("Index Exists");
        *z = reinsert;
    }
}

impl<N, E: Default> TakePutBack<EdgeIndex, EdgeIndex> for Graph<N, E> {
    type ItemType = E;
    type PutType = E;

    fn take(&mut self, index_into: EdgeIndex) -> Self::ItemType {
        let z = self.edge_weight_mut(index_into).expect("Index Exists");
        std::mem::take(z)
    }

    fn put_back(&mut self, index_into: EdgeIndex, reinsert: Self::PutType) {
        let z = self.edge_weight_mut(index_into).expect("Index Exists");
        *z = reinsert;
    }
}

// replace a single node with a graph equipped with two special nodes
// the incoming neighbors of the old node are connected to the first special node
// the outgoing neighbors of the old node are connected from the second special node
impl<N: Default, E: Send + Clone + 'static>
    TakePutBack<NodeIndex, (NodeIndex, Vec<NodeIndex>, Vec<NodeIndex>)> for StableGraph<N, E>
{
    type ItemType = (N, Vec<(NodeIndex, E)>, Vec<(NodeIndex, E)>);
    type PutType = (
        StableGraph<N, E>,
        NodeIndex,
        NodeIndex,
        Vec<(NodeIndex, E)>,
        Vec<(NodeIndex, E)>,
    );

    fn take(&mut self, index_into: NodeIndex) -> Self::ItemType {
        info!("the connections are still present as is, we just removed the weight.
                When it gets put back after processing, the connections will change");
        let z = self.node_weight_mut(index_into).expect("Index Exists");
        let node_weight = std::mem::take(z);
        let incoming_edges = self.edges_directed(index_into, Direction::Incoming);
        let outgoing_edges = self.edges_directed(index_into, Direction::Outgoing);
        let incoming_neighbors = incoming_edges
            .map(|z| (z.source(), z.weight().clone()))
            .collect::<Vec<_>>();
        let outgoing_neighbors = outgoing_edges
            .map(|z| (z.target(), z.weight().clone()))
            .collect::<Vec<_>>();
        (node_weight, incoming_neighbors, outgoing_neighbors)
    }

    fn put_back(
        &mut self,
        index_into: (NodeIndex, Vec<NodeIndex>, Vec<NodeIndex>),
        reinsert: Self::PutType,
    ) {
        #[allow(unused_variables)]
        let (
            replace_graph,
            initial_in_replace,
            final_in_replace,
            in_connections_ambient,
            out_connections_ambient,
        ) = reinsert;
        assert!(replace_graph.contains_node(initial_in_replace));
        assert!(replace_graph.contains_node(final_in_replace));
        let (this_node, incoming_allowed_nodes, outgoing_allowed_nodes) = index_into;
        #[allow(unused_variables)]
        let in_connections_ambient = in_connections_ambient.into_iter().filter(|z| {
            self.contains_edge(z.0, this_node) && incoming_allowed_nodes.contains(&z.0)
        });
        #[allow(unused_variables)]
        let out_connections_ambient = out_connections_ambient.into_iter().filter(|z| {
            self.contains_edge(this_node, z.0) && outgoing_allowed_nodes.contains(&z.0)
        });
        todo!(
            "Remove this node, disjoint union with replacement graph,
                then connect the two special nodes
                (their indices will have shifted when putting into self)
                with above ambient nodes"
        );
    }
}

fn main() {
    println!("Hello World");
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
