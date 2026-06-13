import { useMemo } from "react";
import ReactFlow, { Background, Controls, type Node, type Edge } from "reactflow";
import "reactflow/dist/style.css";
import type { DatabaseMetadata, TableSchema } from "../../types";

interface ErDiagramProps {
  metadata: DatabaseMetadata;
  tableSchemas?: Map<string, TableSchema>;
}

function buildNodes(metadata: DatabaseMetadata): Node[] {
  const tables = metadata.tables.filter(t => t.table_type === "BASE TABLE");
  const colsPerRow = Math.ceil(Math.sqrt(tables.length));
  return tables.map((table, i) => {
    const col = i % colsPerRow;
    const row = Math.floor(i / colsPerRow);
    return {
      id: table.name,
      type: "default",
      position: { x: col * 320, y: row * 250 },
      data: {
        label: (
          <div style={{ fontSize: 12, padding: 4, minWidth: 150 }}>
            <div style={{ fontWeight: 600, borderBottom: "1px solid #ddd", paddingBottom: 4, marginBottom: 4 }}>{table.name}</div>
            <div style={{ color: "#666" }}>{table.row_count ?? "?"} 行</div>
            {table.comment && <div style={{ color: "#999", fontSize: 11 }}>{table.comment}</div>}
          </div>
        ),
      },
      style: { background: "#fff", border: "1px solid #ddd", borderRadius: 6, padding: 0 },
    };
  });
}

function buildEdges(tableSchemas?: Map<string, TableSchema>): Edge[] {
  if (!tableSchemas) return [];
  const edges: Edge[] = [];
  tableSchemas.forEach((schema, _tableName) => {
    schema.constraints.forEach((constraint) => {
      if (constraint.referenced_table && constraint.constraint_type === "FOREIGN KEY") {
        edges.push({
          id: `${schema.table_name}-${constraint.name}`,
          source: schema.table_name,
          target: constraint.referenced_table,
          label: constraint.name,
          animated: true,
          style: { stroke: "#1677ff" },
        });
      }
    });
  });
  return edges;
}

export default function ErDiagram({ metadata, tableSchemas }: ErDiagramProps) {
  const nodes = useMemo(() => buildNodes(metadata), [metadata]);
  const edges = useMemo(() => buildEdges(tableSchemas), [tableSchemas]);

  return (
    <div style={{ width: "100%", height: "100%" }}>
      <ReactFlow nodes={nodes} edges={edges} fitView>
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}
