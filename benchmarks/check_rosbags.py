import argparse
from pathlib import Path
import tqdm

import rosbag
from rosbags.highlevel import AnyReader
from rustbag import Bag
from embag import View


def main():
    parser = argparse.ArgumentParser()

    parser.add_argument("bag", type=str, help="Path to a bag file. If it stored in MinIO it should start with s3://<bucket-name>")
    parser.add_argument("--minio-access-key", type=str, help="Access Key to MinIO", default="minioadmin")
    parser.add_argument("--minio-secret-key", type=str, help="Secret Key to MinIO", default="minioadmin")
    parser.add_argument("--minio-endpoint", type=str, help="MinIO Endpoint", default="http://localhost:9000")
    parser.add_argument("--no-rosbag", action="store_true", help="Skip testing rosbag")
    parser.add_argument("--no-rosbags", action="store_true", help="Skip testing rosbags")
    parser.add_argument("--no-rustbag", action="store_true", help="Skip testing rustbag")
    parser.add_argument("--no-embag", action="store_true", help="Skip testing embag")

    args = parser.parse_args()

    test_libs = set()
    if not args.no_rosbag:
        test_libs.add("rosbag")
    if not args.no_rosbags:
        test_libs.add("rosbags")
    if not args.no_rustbag:
        test_libs.add("rustbag")
    if not args.no_embag:
        test_libs.add("embag")

    verify_bag_path_arg(args.bag, test_libs)

    storage_options = {
        "endpoint": args.minio_endpoint,
        "access_key_id": args.minio_access_key,
        "secret_access_key": args.minio_secret_key,
        "allow_http": "true",
    }

    run_bag_benchmarks(args.bag, storage_options, test_libs)


def verify_bag_path_arg(bag_path: str, test_libs: set[str]):
    if bag_path.startswith("s3://"):
        if len(test_libs) > 0 and test_libs != {"rustbag"}:
            # Check that s3fs is used and mounted
            s3fs_path = s3_path_to_s3fs(bag_path)
            if not s3fs_path.exists():
                raise ValueError("Could not find bag path mounted via s3fs. Please mount the bucket specified in path to <root-of-this-repo>/s3")
    else:
        if not Path(bag_path).exists():
            raise ValueError(f"Could not find bag file at path: {bag_path}")


def s3_path_to_s3fs(path: str) -> Path:
    path_within_bucket = path[len("s3://"):].split("/", 1)[1]
    return Path(__file__).parent.parent / "s3" / path_within_bucket


def run_bag_benchmarks(
    bag_path: str,
    storage_options: dict[str, str],
    test_libs: set[str]
):
    if bag_path.startswith("s3://"):
        local_path = s3_path_to_s3fs(bag_path)
    else:
        local_path = Path(bag_path)

    if "rustbag" in test_libs:
        if bag_path.startswith("s3://"):
            rustbag_path = bag_path
        else:
            storage_options = None
            rustbag_path = f"file://{str(local_path.absolute())}"

        bag = Bag(rustbag_path, storage_options=storage_options)
        for msg in tqdm.tqdm(bag.read_messages(), desc="rustbag"):
            pass

    # create reader instance and open for reading
    with AnyReader([local_path]) as reader:
        for connection, timestamp, rawdata in tqdm.tqdm(reader.messages(), desc="rosbags"):
            msg = reader.deserialize(rawdata, connection.msgtype)

    view = View()
    view.addBag(str(local_path))

    for msg in tqdm.tqdm(view.getMessages(), desc="embag"):
        deser_data = msg.data()
        pass

    bag = rosbag.Bag(str(local_path))
    for msg in tqdm.tqdm(bag.read_messages(), desc="rosbag"):
        pass


if __name__ == "__main__":
    main()

