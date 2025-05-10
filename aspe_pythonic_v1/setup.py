from setuptools import setup, find_packages

setup(
    name="aspe_pythonic_v1",
    version="0.1.0",
    packages=find_packages(),
    install_requires=[
        "click>=8.0.0",
        "pyyacc>=0.3.0",
    ],
    entry_points={
        'console_scripts': [
            'aspe=aspe.cli:cli',
        ],
    },
)