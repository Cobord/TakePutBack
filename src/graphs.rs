use log::info;

use petgraph::{
    prelude::{EdgeIndex, NodeIndex},
    stable_graph::StableGraph,
    visit::EdgeRef,
    Direction, Graph,
};

use crate::general::TakePutBack;

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

    fn all_idces_inout(&self) -> Vec<(NodeIndex, NodeIndex)> {
        self.node_indices().map(|z| (z, z)).collect()
    }

    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType {
        |z| {
            info!("Doing nothing on node of stable graph");
            z
        }
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

    fn all_idces_inout(&self) -> Vec<(NodeIndex, NodeIndex)> {
        self.node_indices().map(|z| (z, z)).collect()
    }

    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType {
        |z| {
            info!("Doing nothing on node of unstable graph");
            z
        }
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

    fn all_idces_inout(&self) -> Vec<(EdgeIndex, EdgeIndex)> {
        self.edge_indices().map(|z| (z, z)).collect()
    }

    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType {
        |z| {
            info!("Doing nothing on edge of stable graph");
            z
        }
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

    fn all_idces_inout(&self) -> Vec<(EdgeIndex, EdgeIndex)> {
        self.edge_indices().map(|z| (z, z)).collect()
    }

    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType {
        |z| {
            info!("Doing nothing on edge of unstable graph");
            z
        }
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
        info!(
            "the connections are still present as is, we just removed the weight.
                When it gets put back after processing, the connections will change"
        );
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

    fn all_idces_inout(&self) -> Vec<(NodeIndex, (NodeIndex, Vec<NodeIndex>, Vec<NodeIndex>))> {
        self.node_indices()
            .map(|z| {
                (
                    z,
                    (
                        z,
                        self.neighbors_directed(z, Direction::Incoming).collect(),
                        self.neighbors_directed(z, Direction::Outgoing).collect(),
                    ),
                )
            })
            .collect()
    }

    fn do_nothing_process(&self) -> fn(Self::ItemType) -> Self::PutType {
        |(node_data, in_edges, out_edges)| {
            let mut ret_graph = StableGraph::new();
            let ret_idx = ret_graph.add_node(node_data);
            (ret_graph, ret_idx, ret_idx, in_edges, out_edges)
        }
    }
}

mod test {
    #[test]
    fn presence() {
        #[allow(unused_imports)]
        use super::*;
        // just make sure the above functions are present
    }
}
