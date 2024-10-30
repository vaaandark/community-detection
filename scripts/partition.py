#!/usr/bin/env python3

import networkx as nx
import matplotlib.pyplot as plt
import sys
import os
import numpy as np

if len(sys.argv) != 2:
    print(f"usage: f{sys.argv[0]} path")
    exit(1)

path = sys.argv[1]
G = nx.read_edgelist(path, data=False)

communities = nx.community.louvain_communities(G)

ncommunities = len(communities)
q = nx.community.modularity(G, nx.community.label_propagation_communities(G))
print(f"graph: communities={ncommunities}, modularity={q}")

if "DRAW" in os.environ:
    # Compute positions for the node clusters as if they were themselves nodes in a
    # supergraph using a larger scale factor
    supergraph = nx.cycle_graph(len(communities))
    superpos = nx.spring_layout(G, scale=50, seed=429)

    # Use the "supernode" positions as the center of each node cluster
    centers = list(superpos.values())
    pos = {}
    for center, comm in zip(centers, communities):
        pos.update(nx.spring_layout(nx.subgraph(G, comm), center=center, seed=1430))

    cmap = plt.get_cmap("tab20")
    colors = [cmap(i) for i in np.linspace(0, 1, ncommunities)]
    for i, nodes in enumerate(communities):
        nx.draw_networkx_nodes(
            G, pos, nodelist=nodes, node_size=20, node_color=colors[i]
        )
    nx.draw_networkx_edges(G, pos, alpha=0.5)
    plt.tight_layout()
    plt.show()
