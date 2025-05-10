"""
Tests for ASPE Pythonic to verify translation works both ways.
"""
import json
from aspe.pythonic_to_asg import pythonic_to_asg
from aspe.asg_to_pythonic import asg_to_pythonic

def test_basic_expression():
    # Simple arithmetic
    py_code = "2 + 3 * 4"
    asg_json = pythonic_to_asg(py_code)
    
    # Verify JSON is valid
    asg = json.loads(asg_json)
    assert "nodes" in asg
    assert "root_node_id" in asg
    
    # Convert back to Python (may not be identical due to formatting)
    py_result = asg_to_pythonic(asg_json)
    # Basic check: should contain the numbers
    assert "2" in py_result
    assert "3" in py_result
    assert "4" in py_result
    
def test_lambda_expression():
    # Lambda function
    py_code = "lambda x: x + 1"
    asg_json = pythonic_to_asg(py_code)
    
    # Verify JSON structure
    asg = json.loads(asg_json)
    assert "nodes" in asg
    assert "root_node_id" in asg
    
    # Find lambda node
    lambda_node = None
    for node_id, node in asg["nodes"].items():
        if node.get("type_") == "TermLambda":
            lambda_node = node
            break
    
    assert lambda_node is not None
    assert "binder_variable_node_id" in lambda_node
    assert "body_node_id" in lambda_node
    
    # Convert back to Python
    py_result = asg_to_pythonic(asg_json)
    assert "lambda" in py_result
    assert "x" in py_result
    assert "+" in py_result
    assert "1" in py_result

if __name__ == "__main__":
    test_basic_expression()
    test_lambda_expression()
    print("All tests passed!")