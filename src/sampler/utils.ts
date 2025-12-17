/**
 * Chain together audio nodes
 */
export function chain(...nodes: AudioNode[]) {
  for (let i = 0; i < nodes.length - 1; i++) {
    const node = nodes[i];
    node.connect(nodes[i + 1]);
  }
}
