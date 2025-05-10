"""
Converts Synapse ASG to Pythonic Python code.
Translates the abstract semantics to idiomatic Python syntax.
"""

import json
from typing import Dict, Any, List, Optional, cast

class PythonicCodeGenerator:
    """
    Generates Pythonic Python code from Synapse ASG.
    """
    
    def __init__(self, asg_json: str):
        """Initialize with a Synapse ASG JSON string"""
        try:
            self.asg = json.loads(asg_json)
            self.nodes = self.asg.get("nodes", {})
            self.root_id = self.asg.get("root_node_id")
        except json.JSONDecodeError:
            self.asg = {}
            self.nodes = {}
            self.root_id = None
            
    def generate(self) -> str:
        """Generate Pythonic Python code from the ASG"""
        if not self.root_id or self.root_id not in self.nodes:
            return "# Empty or invalid ASG"
            
        return self._generate_node(self.root_id)
        
    def _generate_node(self, node_id: int) -> str:
        """Generate code for a specific ASG node"""
        node = self.nodes.get(str(node_id))
        if not node:
            return "# Missing node"
            
        node_type = node.get("type_", "Unknown")
        
        if node_type == "LiteralInt":
            return str(node.get("value", 0))
            
        elif node_type == "TermVariable":
            return node.get("name", "undefined")
            
        elif node_type == "PrimitiveOp":
            op_name = node.get("op_name", "unknown")
            arg_ids = node.get("argument_node_ids", [])
            
            if len(arg_ids) != 2:
                return f"# Invalid {op_name} operation"
                
            left = self._generate_node(arg_ids[0])
            right = self._generate_node(arg_ids[1])
            
            if op_name == "add":
                return f"({left} + {right})"
            elif op_name == "sub":
                return f"({left} - {right})"
            elif op_name == "mul":
                return f"({left} * {right})"
            elif op_name == "div":
                return f"({left} / {right})"
            else:
                return f"({left} {op_name} {right})"
                
        elif node_type == "TermLambda":
            binder_id = node.get("binder_variable_node_id")
            body_id = node.get("body_node_id")
            
            if not binder_id or not body_id:
                return "lambda: ..."
                
            # Get parameter name
            binder_node = self.nodes.get(str(binder_id))
            param_name = binder_node.get("name", "x") if binder_node else "x"
            
            # Generate body code
            body_code = self._generate_node(body_id)
            
            return f"lambda {param_name}: {body_code}"
            
        else:
            return f"# Unsupported node type: {node_type}"

def asg_to_pythonic(asg_json: str) -> str:
    """
    Convert Synapse ASG to Pythonic Python code.
    """
    generator = PythonicCodeGenerator(asg_json)
    return generator.generate()