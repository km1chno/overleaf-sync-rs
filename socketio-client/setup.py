from setuptools import setup, find_packages

setup(
    name="olsync-rs-socketio-client",
    version="0.1.0",
    packages=find_packages(),
    entry_points={
        'console_scripts': [
            'olsync-rs-socketio-client = src.client:main',
        ],
    },
    author="Katzper Michno",
    author_email="katzper.michno@gmail.com",
    description="overleaf-sync-rs socket.io client",
    python_requires='>=3.6',
)

