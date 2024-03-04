# Getting Started

## Installation
Available on PyPI: `pip install rustbag`.
To run below examples you will also need [tqdm]() for a neat progress bar (`pip install tqdm`).

## Basic Usage

### Reading a local file
```python
from rustbag import Bag
import tqdm


path = Path("<bag-path>.bag")
bag = Bag(f"file://{str(path.absolute())}")
for msg in tqdm.tqdm(bag.read_messages(), desc="rs", total=bag.num_messages()):
    pass
```

### Reading a file from local S3 deployment
It is recommended to be read a bag through a wired interface due to lower latency, higher bandwidth and more stable connection.

```python
from rustbag import Bag
import tqdm

bag_name = "<bag-path>.bag"
bag = Bag(f"s3://<bucket-name>/{bag_name}", {
    # NOTE: Replace below with actual values, or checkout benchmarks/ folder on GitHub for standing up playground MinIO
    "endpoint": "http://localhost:9000",
    "access_key_id": "minioadmin",
    "secret_access_key": "minioadmin",
    "allow_http": "true",
})
for msg in tqdm.tqdm(bag.read_messages(), desc="rs", total=bag.num_messages()):
    pass
```


### Reading a file from AWS S3 deployment
It is recommended to be read a bag through a wired interface due to lower latency, higher bandwidth and more stable connection.

!!! warning

    I do not recommend reading bags directly from AWS (or other cloud provider) like this due to significantly higher costs, then just downloading and reading the bag from a local machine. This is **NOT** what RustBag was designed to do.

```python
from rustbag import Bag
import tqdm

bag_name = "<bag-path>.bag"
bag = Bag(f"s3://bucket-name/{bag_name}", {
    # NOTE: Replace below with actual values
    "access_key_id": "<grab-from-aws>",
    "secret_access_key": "<grab-from-aws>",
})
for msg in tqdm.tqdm(bag.read_messages(), desc="rs", total=bag.num_messages()):
    pass
```

