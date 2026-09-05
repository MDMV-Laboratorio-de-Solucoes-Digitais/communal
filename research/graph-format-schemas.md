# Graph File Format Schema Definitions — Research Report

A comprehensive reference for graph file format specifications commonly used in network analysis, covering EdgeList, JSON-based formats, GML, and SNAP dataset formats. Includes validation rules and constraints for each.

---

## Table of Contents

1. [EdgeList Format](#1-edgelist-format)
2. [JSON Graph Formats](#2-json-graph-formats)
   - [2.1 Graphology JSON Format](#21-graphology-json-format)
   - [2.2 NetworkX Node-Link Format](#22-networkx-node-link-format)
   - [2.3 JSON Graph Format (JGF)](#23-json-graph-format-jgf)
   - [2.4 D3-Force / D3.js Format](#24-d3-force--d3js-format)
3. [GML — Graph Modelling Language](#3-gml--graph-modelling-language)
4. [SNAP Dataset Format](#4-snap-dataset-format)
5. [Summary of Validation Rules and Constraints](#5-summary-of-validation-rules-and-constraints)

---

## 1. EdgeList Format

### 1.1 Overview

The EdgeList is one of the simplest graph representation formats, storing a graph as a list of node pairs. It is widely used in network analysis tools due to its simplicity and compactness. There is no standardized formal specification; the format is defined by the tools that implement it (NetworkX, NetworKit, SNAP).

### 1.2 Variants

#### 1.2.1 Unweighted, Unlabeled EdgeList

Each line contains two node identifiers representing an edge:

```
1 2
2 3
3 1
```

#### 1.2.2 Weighted EdgeList (NetworkX `read_weighted_edgelist`)

Each line contains two node identifiers followed by a numeric weight:

```
1 2 0.5
2 3 1.0
3 1 2.3
```

The weight is stored as a float under the key `"weight"`. Nodes must be hashable types. [1]

#### 1.2.3 EdgeList with Dictionary Data (NetworkX `read_edgelist`)

Each line can contain two node identifiers followed by a Python dictionary of edge attributes:

```
1 2 {'weight': 7, 'color': 'green'}
```

#### 1.2.4 Arbitrary Data EdgeList (NetworkX)

Each line can contain two node identifiers followed by arbitrary space-separated values:

```
1 2 7 green
```

### 1.3 Directed vs. Undirected

The EdgeList format itself does not encode directedness. Directionality is determined by the reader/writer configuration:

- **NetworkX**: The `create_using` parameter controls graph type (`nx.Graph` for undirected, `nx.DiGraph` for directed). By default, undirected. [1]
- **NetworKit**: The `directed` parameter on `EdgeListReader` controls this (default `False`). [2]

### 1.4 Validation Rules

| Rule | Constraint |
|------|-----------|
| Node count per line | Exactly 2 nodes required per edge line |
| Isolated nodes | Cannot be represented unless they have a self-loop |
| Comments | Lines starting with `#` are ignored (configurable in NetworkX via `comments` parameter) |
| Delimiter | Whitespace by default; configurable (any string) |
| Node type | Must be hashable (configurable via `nodetype` parameter in NetworkX) |
| Weight | Must be parseable as a float (for weighted variant) |
| Self-loops | Allowed (node paired with itself) |

---

## 2. JSON Graph Formats

### 2.1 Graphology JSON Format

#### 2.1.1 Overview

Graphology is a robust multipurpose Graph library for JavaScript/TypeScript. Its JSON serialization format is a structured object supporting directed, undirected, and mixed graphs with self-loops and parallel edges. [3]

#### 2.1.2 Schema

**Top-level structure:**

```jsonc
{
  "attributes": { /* graph-level attributes */ },
  "options": {
    "allowSelfLoops": boolean,  // default: true
    "multi": boolean,           // default: false (parallel edges)
    "type": "directed" | "undirected" | "mixed"  // default: "directed"
  },
  "nodes": [ /* list of serialized nodes */ ],
  "edges": [ /* list of serialized edges */ ]
}
```

**Serialized Node:**

```jsonc
{
  "key": any,           // The node's key (required)
  "attributes": { }     // The node's attributes (optional, can be null)
}
```

**Serialized Edge:**

```jsonc
{
  "key": any,           // The edge's key (optional on import)
  "source": any,        // The edge's source node key (required)
  "target": any,        // The edge's target node key (required)
  "attributes": { },    // The edge's attributes (optional, can be null)
  "undirected": boolean // Whether the edge is undirected (optional)
}
```

#### 2.1.3 Validation Rules

| Rule | Constraint |
|------|-----------|
| Node key | Required; must be unique within the graph |
| Edge source/target | Must reference existing node keys |
| Edge key | Optional on import; auto-generated if omitted |
| `options.type` | Must be one of `"directed"`, `"undirected"`, `"mixed"` |
| `options.multi` | Boolean; `true` allows parallel edges |
| `options.allowSelfLoops` | Boolean; `true` allows edges where source === target |
| Graph attributes | Optional object; can be omitted entirely |
| Nodes array | Can be omitted when merging into existing graph |
| Edges array | Can be omitted when merging into existing graph |

### 2.2 NetworkX Node-Link Format

#### 2.2.1 Overview

The `node_link_data` function in NetworkX produces a JSON-serializable dictionary representing the graph's structure. This format is designed for use with JavaScript visualization libraries. [4][5]

#### 2.2.2 Schema

```jsonc
{
  "directed": boolean,       // true if graph is directed
  "multigraph": boolean,     // true if graph supports parallel edges
  "graph": { },              // graph-level attributes (dictionary)
  "nodes": [                 // list of node objects
    {
      "id": any,             // node identifier
      /* arbitrary attribute keys */
    }
  ],
  "links": [                 // list of edge objects
    {
      "source": any,         // source node id
      "target": any,         // target node id
      "key": any,            // edge key (only for multigraphs)
      /* arbitrary attribute keys */
    }
  ]
}
```

#### 2.2.3 Validation Rules

| Rule | Constraint |
|------|-----------|
| `directed` | Boolean; defaults to `false` |
| `multigraph` | Boolean; `true` enables parallel edges |
| `nodes` | Must be an array; each node must have an `"id"` key |
| `links` | Must be an array; each link must have `"source"` and `"target"` |
| `key` | Required in links only when `multigraph` is `true` |
| Attribute keys | Converted to strings (JSON requirement) |
| Graph attributes | Stored in the `"graph"` object |

### 2.3 JSON Graph Format (JGF)

#### 2.3.1 Overview

JSON Graph Format (JGF) is a standardized proposal for representing graph structures in JSON, maintained by the `jsongraph` community. Version 2 is the current specification. It supports single graphs and multi-graph collections, hyperedges, and metadata. [6]

#### 2.3.2 Schema (Version 2)

**Single Graph:**

```jsonc
{
  "graph": {
    "id": "string",              // optional graph identifier
    "type": "string",            // graph classification
    "label": "string",           // display label
    "directed": boolean,         // true = directed (default), false = undirected
    "metadata": { },             // custom graph-level metadata
    "nodes": {                   // Map/Dictionary of node objects
      "<node-id>": {
        "label": "string",       // display label
        "metadata": { }          // custom node metadata
      }
    },
    "edges": [                   // array of edge objects
      {
        "source": "string",      // key of source node
        "target": "string",      // key of target node
        "relation": "string",    // interaction type
        "directed": boolean,     // edge direction (defaults to graph.directed)
        "label": "string",       // edge display label
        "metadata": { }          // custom edge metadata
      }
    ],
    "hyperedges": [              // optional hyperedge array
      {
        "nodes": ["string"],     // array of node keys (undirected hyperedge)
        "source": ["string"],    // array of source node keys (directed hyperedge)
        "target": ["string"],    // array of target node keys (directed hyperedge)
        "relation": "string",    // interaction type
        "metadata": { }          // custom hyperedge metadata
      }
    ]
  }
}
```

**Multi-Graph Collection:**

```jsonc
{
  "graphs": [
    { /* graph object */ },
    { /* graph object */ }
  ]
}
```

#### 2.3.3 Validation Rules

| Rule | Constraint |
|------|-----------|
| `nodes` | Must be a Map/Dictionary (not array); each key is a unique node identifier |
| `edges` | Must be an array; `source` and `target` must reference valid node keys |
| `directed` | Defaults to `true` at graph level; can be overridden per-edge |
| `metadata` | Optional object on graph, node, and edge levels |
| Hyperedges | Either undirected (`nodes` array) or directed (`source`/`target` arrays), not both in same graph |
| Media type | `application/vnd.jgf+json` |
| JSON Schema | Formal schema provided at `json-graph-schema_v2.json` [6] |
| Null values | Properties allowing `null` may be omitted entirely |

### 2.4 D3-Force / D3.js Format

#### 2.4.1 Overview

D3.js uses a de facto JSON format for force-directed graph layouts. There is no formal specification, but the format is consistent across D3 examples and the `d3-force` library. [7]

#### 2.4.2 Schema

```jsonc
{
  "nodes": [
    { "id": "string", /* optional attributes: x, y, fx, fy, vx, vy, group, weight, etc. */ },
    { "id": "string" }
  ],
  "links": [
    { "source": "node-id-or-index", "target": "node-id-or-index", /* optional: value, distance */ },
    { "source": "...", "target": "..." }
  ]
}
```

#### 2.4.3 Validation Rules

| Rule | Constraint |
|------|-----------|
| `nodes` | Array of objects; each should have an `id` or be referenced by index |
| `links` | Array of objects; each must have `source` and `target` |
| Link references | `source` and `target` can be string IDs matching node `id`, or numeric indices into the nodes array |
| Self-loops | Allowed (source === target) |
| Parallel edges | Allowed (duplicate source/target pairs) |
| Node positioning | Optional `x`, `y`, `fx` (fixed x), `fy` (fixed y) attributes |
| Link distance | Optional; overrides force link distance accessor |

---

## 3. GML — Graph Modelling Language

### 3.1 Overview

GML (Graph Modelling Language, also called Graph Meta Language) is a hierarchical ASCII-based file format for describing graphs. Developed by Michael Himsolt and first proposed at GD'95, it is the standard file format for the Graphlet editor and is supported by many network analysis tools including NetworkX, Cytoscape, Gephi, igraph, and yEd. [8][9]

**Media type:** `text/vnd.gml`
**File extension:** `.gml`

### 3.2 Formal Grammar (BNF)

From the official technical report: [8]

```
GML          ::= List
List         ::= (whitespace* Key whitespace+ Value)*
Value        ::= Integer | Real | String | [ List ]
Key          ::= [a-zA-Z][a-zA-Z0-9]*
Integer      ::= sign digit+
Real         ::= sign digit*.digit* mantissa
String       ::= '"' instring '"'
sign         ::= empty | '+' | '-'
digit        ::= [0-9]
mantissa     ::= empty | 'E' sign digit+
instring     ::= ASCII-{"\\&} | '&' character ';'
whitespace   ::= space | tabulator | newline
```

### 3.3 Structure

A GML file is a tree of key-value pairs. The top-level key must be `graph`. Inside:

```
graph [
  comment "This is a sample graph"
  directed 1
  id 42
  label "Hello, I am a graph"
  node [
    id 1
    label "node 1"
    thisIsASampleAttribute 42
  ]
  node [
    id 2
    label "node 2"
  ]
  edge [
    source 1
    target 2
    label "Edge from node 1 to node 2"
  ]
]
```

### 3.4 Global Defined Keys

From the official specification: [8]

| Key | Type | Description |
|-----|------|-------------|
| `.id` | int | Identification number for an object (used for pointers) |
| `.label` | string | Label attached to an object |
| `.comment` | string | Comment embedded in the file (ignored by application) |
| `.Creator` | string | Application that created the file (top-level only) |
| `.graphics` | list | Graphics information for drawing an object |

### 3.5 Graph Keys

| Key | Type | Description |
|-----|------|-------------|
| `.graph` | list | Describes a graph |
| `.directed` | int | 1 = directed, 0 = undirected; default is undirected (0) |
| `.graph.node` | list | Describes a node; non-isolated nodes must have `.graph.node.id` |
| `.graph.edge` | list | Describes an edge |
| `.graph.edge.source` | int | id of source node |
| `.graph.edge.target` | int | id of target node |

### 3.6 Validation Rules

| Rule | Constraint |
|------|-----------|
| Character encoding | 7-bit ASCII only (ISO 8859-1 for extended characters, using `&name;` encoding) |
| Maximum key size | 254 characters |
| Maximum line length | 254 characters |
| Comments | Lines starting with `#` are ignored |
| Node id uniqueness | `.graph.node.id` values must be unique within the graph |
| Edge endpoints | Every edge must have both `.graph.edge.source` and `.graph.edge.target` |
| Directed/undirected | Controlled by `.graph.directed` (0 or 1); default undirected |
| Unsafe keys | Keys starting with a capital letter are considered invalid after any graph modification |
| Unknown attributes | Applications must preserve unknown attributes and re-write them |
| Order significance | Order is not significant unless multiple entries share the same key |
| String escaping | `"` and `&` within strings must be encoded as `&quot;` and `&amp;` |
| Integer range | Signed 32-bit integers; larger numbers should be strings |
| Floating point | Must be within double-precision float range |
| Boolean values | Represented as 0 (false) and 1 (true) |
| Isolated nodes | Can exist without `id` field (only non-isolated nodes require `id`) |

### 3.7 Directed vs. Undirected

Directed and undirected graphs use the same format. The distinction is made via the `.graph.directed` attribute:
- `directed 1` → directed graph
- `directed 0` or omitted → undirected graph

In undirected graphs, `source` and `target` may be assigned arbitrarily (though they may imply visual direction for polyline edges). [8]

---

## 4. SNAP Dataset Format

### 4.1 Overview

The SNAP (Stanford Network Analysis Platform) dataset format is used for the Stanford Large Network Dataset Collection and the SNAP library. It is a simple edge-list variant designed for efficient storage and reading of large networks. [2][10]

### 4.2 Format Specification

**Optional Problem Line:**

The first line optionally denotes the problem line:

```
p <0 or 1-indexed>
```

**Edge List:**

Following the problem line (if present), each line represents one edge:

- **Unweighted:** `<u v>`
- **Weighted:** `<u v w>`

Where:
- `u` = source node ID
- `v` = target node ID
- `w` = edge weight (float, weighted graphs only)

### 4.3 Configuration Parameters (NetworKit `SNAPGraphReader`)

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `directed` | bool | `False` | Whether the graph is directed |
| `remapNodes` | bool | `True` | Remap node IDs to consecutive integers |
| `nodeCount` | int | `0` | Number of nodes for preallocation |

### 4.4 Validation Rules

| Rule | Constraint |
|------|-----------|
| Problem line | Optional; must start with `p` if present |
| Edge format | Each edge line must have exactly 2 (unweighted) or 3 (weighted) columns |
| Node IDs | Integer values; 0-indexed or 1-indexed (per problem line) |
| Weight | Must be parseable as a numeric value (float) |
| Directedness | Controlled by reader parameter, not encoded in the file |
| Comments | Lines starting with `#` are ignored (standard SNAP convention) |
| Self-loops | Allowed |
| Duplicate edges | Depends on reader configuration |

### 4.5 SNAP EdgeList Variants (NetworKit)

NetworKit provides several standard EdgeListReader configurations: [2]

| Variant | Separator | First Node ID |
|---------|-----------|---------------|
| `EdgeListSpaceZero` | whitespace | 0 |
| `EdgeListSpaceOne` | whitespace | 1 |
| `EdgeListTabZero` | tab | 0 |
| `EdgeListTabOne` | tab | 1 |
| `EdgeListCommaOne` | comma | 1 |

---

## 5. Summary of Validation Rules and Constraints

### Cross-Format Comparison

| Feature | EdgeList | Graphology JSON | NetworkX Node-Link | JGF | GML | SNAP |
|---------|----------|-----------------|-------------------|-----|-----|------|
| **Directed** | External flag | `options.type` | `directed` field | `directed` field | `.directed` 0/1 | External flag |
| **Weighted** | 3rd column | Edge attributes | Link attributes | Edge metadata | Any key | 3rd column |
| **Self-loops** | Allowed | `allowSelfLoops` option | Allowed | Allowed | Allowed | Allowed |
| **Parallel edges** | Duplicates | `multi` option | `multigraph` + `key` | Allowed | Allowed | Depends |
| **Node attributes** | Not supported | `node.attributes` | Node object attrs | Node `metadata` | Any node key | Not supported |
| **Edge attributes** | Limited | `edge.attributes` | Link object attrs | Edge `metadata` | Any edge key | Weight only |
| **Graph attributes** | Not supported | `attributes` field | `graph` object | Graph `metadata` | Any graph key | Not supported |
| **Isolated nodes** | Not representable | Yes | Yes | Yes | Yes | Not representable |
| **Comments** | `#` prefix | N/A (JSON) | N/A (JSON) | N/A (JSON) | `#` prefix | `#` prefix |
| **Encoding** | UTF-8/ASCII | JSON (UTF-8) | JSON (UTF-8) | JSON (UTF-8) | 7-bit ASCII | UTF-8/ASCII |

### Key Constraints by Format

1. **EdgeList/SNAP**: Minimal format — no node/graph metadata, no isolated nodes without self-loops. Directionality is external. Fastest to parse.

2. **Graphology JSON**: Rich format supporting all graph types. Requires `key` for each node. Edge `source`/`target` must reference valid node keys. The `options` block defines graph behavior.

3. **NetworkX Node-Link**: JSON-serializable dictionary. `nodes` array with `id`; `links` array with `source`/`target`. Graph attributes in `graph` object. `multigraph` flag controls parallel edge support.

4. **JSON Graph Format (JGF)**: Standardized with formal JSON Schema. Nodes as a Map (not array). Supports hyperedges. Can represent collections of graphs. Most semantically structured JSON format.

5. **GML**: Hierarchical and extensible. Requires unique node `id` values. Directedness via `.directed`. Supports arbitrary nested attributes. 7-bit ASCII constraint. Capital-letter keys marked unsafe after modifications.

6. **SNAP**: Minimal edge list with optional problem line. Weighted/unweighted variants. No native attribute support. Optimized for large-scale network datasets.

---

## References

1. **NetworkX EdgeList Documentation** — https://networkx.org/documentation/stable/reference/readwrite/edgelist.html
2. **NetworKit Graph I/O Tutorial** — https://networkit.github.io/dev-docs/notebooks/IONotebook.html
3. **Graphology Serialization Documentation** — https://graphology.github.io/serialization.html (archived at https://github.com/aiminnovations/devkit-graphology/blob/master/docs/serialization.md)
4. **NetworkX node_link_data Documentation** — https://networkx.org/documentation/stable/reference/readwrite/generated/networkx.readwrite.json_graph.node_link_data.html
5. **NetworkX JSON Graph Documentation** — https://networkx.org/documentation/stable/reference/readwrite/json_graph.html
6. **JSON Graph Format Specification (JGF)** — https://github.com/jsongraph/json-graph-specification
7. **D3-Force Documentation** — https://github.com/d3/d3-force
8. **GML: A Portable Graph File Format (Technical Report)** — Michael Himsolt, University of Passau — https://raw.githubusercontent.com/GunterMueller/UNI_PASSAU_FMI_Graph_Drawing/master/GML/gml-technical-report.pdf
9. **GML Wikipedia Article** — https://en.wikipedia.org/wiki/Graph_Modelling_Language
10. **SNAP: Stanford Network Analysis Platform** — https://snap.stanford.edu/

---

*Report compiled: 2026-04-28*
