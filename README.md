A common abstraction of the process of taking some piece out, processing it and then putting it back.

Assuming these operations being independent (any side effects should not matter how they are interleaved), there is a default implementation of process_all as threaded execution of multiple of these. If doing this, you only provide the simpler take and put back implementations.
This means this is assuming that this processing step is the intense part, so that there is a gain from sending each chunk to be done on it's own thread before putting it back.

Do this on all the pieces of a collection. Then we are thinking fmap and the f is of type T->T. The way you index for taking out and putting back are the same, but that is not the case in general.
These are the vector examples and the ones which are changing the data on the nodes/edges of a graph.

There is also an example with graphs were taking out a node and putting in a new graph. This is a case to show where the types used for taking something out and for putting something back in are different.
