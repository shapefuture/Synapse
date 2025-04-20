# (placeholder for ASPE Phase 3 prototype)
# Sample usage: python project_to_asg.py input.py > out.asg

import sys
def project_to_asg(python_code):
    # TODO: Use trained transformer to translate code to S-exp/ASG
    return "(lambda (x : Int) x)"

if __name__ == "__main__":
    pycode = sys.stdin.read()
    print(project_to_asg(pycode))