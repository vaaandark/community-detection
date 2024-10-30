#!/usr/bin/env python3

import sys
import networkx as nx

if len(sys.argv) == 3 or len(sys.argv) == 4:
    n = int(sys.argv[1])
    p = float(sys.argv[2])
    G = nx.erdos_renyi_graph(n, p)
    path = "edgelist.txt"
    if len(sys.argv) == 4:
        path = sys.argv[3]
    nx.write_edgelist(G, path, data=False)
else:
    print(f"usage: {sys.argv[0]} n p [path]")
    print(f"    n: The number of nodes")
    print(f"    p: Probability for edge creation")
    print(f"    path(optional): File or filename to write")
    exit(1)
