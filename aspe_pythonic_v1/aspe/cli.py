"""
Command-line interface for ASPE Pythonic.
"""

import click
import sys
from .pythonic_to_asg import pythonic_to_asg
from .asg_to_pythonic import asg_to_pythonic

@click.group()
def cli():
    """ASPE: AI Syntax Projection Engine for Synapse - Pythonic Edition"""
    pass

@cli.command()
@click.argument('input_file', type=click.File('r'), default='-')
@click.argument('output_file', type=click.File('w'), default='-')
def to_asg(input_file, output_file):
    """Convert Pythonic Python code to Synapse ASG"""
    python_code = input_file.read()
    asg_json = pythonic_to_asg(python_code)
    output_file.write(asg_json)

@cli.command()
@click.argument('input_file', type=click.File('r'), default='-')
@click.argument('output_file', type=click.File('w'), default='-')
def to_pythonic(input_file, output_file):
    """Convert Synapse ASG to Pythonic Python code"""
    asg_json = input_file.read()
    python_code = asg_to_pythonic(asg_json)
    output_file.write(python_code)

if __name__ == '__main__':
    cli()