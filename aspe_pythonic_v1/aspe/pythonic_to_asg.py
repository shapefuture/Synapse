"""
Converts Pythonic Python code to Synapse ASG representation.
Uses Python's ast module to generate a Synapse-compatible ASG structure.
"""

import ast
import json
from typing import Dict, Optional, Any, List, Tuple, cast

class SynapseAsgBuilder:
    """
    Builds Synapse ASG from Python AST.
    Simple demonstration for basic expressions, variables, and functions.
    """
    
    def __init__(self):
        self.nodes = {}
        self.next_id = 1
        
    def _fresh_id(self) -> int:
        """Generate a fresh node ID"""
        node_id = self.next_id
        self.next_id += 1
        return node_id
        
    def _add_node(self, node_type: str, content: Dict[str, Any]) -> int:
        """Add a node to the ASG"""
        node_id = self._fresh_id()
        self.nodes[node_id] = {
            "node_id": node_id,
            "type_": node_type,
            **content
        }
        return node_id
        
    def build_from_src(self, python_code: str) -> Dict[str, Any]:
        """
        Build ASG from Python source code.
        Returns a JSON-serializable dictionary representing the ASG.
        """
        try:
            tree = ast.parse(python_code)
            # Root is usually a Module with a body
            root_id = self._translate_module(tree)
            return {
                "nodes": self.nodes,
                "root_node_id": root_id
            }
        except SyntaxError as e:
            # Handle parsing errors
            print(f"Syntax error: {e}")
            return {"nodes": {}, "root_node_id": None}
            
    def _translate_module(self, module: ast.Module) -> int:
        """Translate a Python module to Synapse ASG"""
        # For simplicity, just translate the last expression/statement
        # (Real implementation would handle multiple statements)
        if not module.body:
            return self._add_node("Unknown", {})
            
        last_node = module.body[-1]
        return self._translate_node(last_node)
        
    def _translate_node(self, node: ast.AST) -> int:
        """Translate a Python AST node to Synapse ASG node"""
        if isinstance(node, ast.Expr):
            return self._translate_node(node.value)
        elif isinstance(node, ast.Num):
            # Integer literal
            return self._add_node("LiteralInt", {"value": node.n})
        elif isinstance(node, ast.Constant) and isinstance(node.value, int):
            # Integer literal (Python 3.8+)
            return self._add_node("LiteralInt", {"value": node.value})
        elif isinstance(node, ast.Name):
            # Variable reference
            return self._add_node("TermVariable", {"name": node.id, "definition_node_id": 0})
        elif isinstance(node, ast.BinOp):
            # Binary operation
            left_id = self._translate_node(node.left)
            right_id = self._translate_node(node.right)
            
            if isinstance(node.op, ast.Add):
                op_name = "add"
            elif isinstance(node.op, ast.Sub):
                op_name = "sub"
            elif isinstance(node.op, ast.Mult):
                op_name = "mul"
            elif isinstance(node.op, ast.Div):
                op_name = "div"
            else:
                op_name = "unknown"
                
            return self._add_node("PrimitiveOp", {
                "op_name": op_name,
                "argument_node_ids": [left_id, right_id]
            })
        elif isinstance(node, ast.Lambda):
            # Lambda expression
            param_names = [param.arg for param in node.args.args]
            
            # Create variable nodes for params
            param_ids = []
            for name in param_names:
                param_id = self._add_node("TermVariable", {"name": name, "definition_node_id": 0})
                param_ids.append(param_id)
                
            # Translate the body with parameters in scope
            body_id = self._translate_node(node.body)
            
            # Create lambda node
            return self._add_node("TermLambda", {
                "binder_variable_node_id": param_ids[0] if param_ids else 0,
                "body_node_id": body_id
            })
        else:
            # Unsupported node
            return self._add_node("Unknown", {})

def pythonic_to_asg(python_code: str) -> str:
    """
    Convert Pythonic Python code to Synapse ASG S-expression format.
    Returns the ASG as a JSON string.
    """
    builder = SynapseAsgBuilder()
    asg = builder.build_from_src(python_code)
    return json.dumps(asg, indent=2)