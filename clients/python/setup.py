from setuptools import setup, find_packages

setup(
    name="ga-core-client",
    version="0.1.0",
    description="Python client for ga-core",
    py_modules=["ga_core_client"],
    install_requires=[
        "requests>=2.28",
    ],
    python_requires=">=3.10",
)
