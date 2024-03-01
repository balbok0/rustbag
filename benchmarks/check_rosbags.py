from pathlib import Path
import tqdm

import rosbag
from rosbags.highlevel import AnyReader
from rosbags_rs import Bag
from embag import View

bag_name = "20210828_2.bag"

path = Path(__file__).parent.parent / "s3" / bag_name
bag = Bag(f"s3://test-bags/{bag_name}", {
    "endpoint": "http://localhost:9000",
    "access_key_id": "minioadmin",
    "secret_access_key": "minioadmin",
    "allow_http": "true",
})
for msg in tqdm.tqdm(bag.read_messages(["/odometry/filtered_odom", "/cmd", "/controls"]), desc="rs"):
    pass

# create reader instance and open for reading
with AnyReader([path]) as reader:
    for connection, timestamp, rawdata in tqdm.tqdm(reader.messages(), desc="rosbags"):
        msg = reader.deserialize(rawdata, connection.msgtype)

view = View()
view.addBag(str(path))

for msg in tqdm.tqdm(view.getMessages(), desc="embag"):
    deser_data = msg.data()
    pass

bag = rosbag.Bag(str(path))
for msg in tqdm.tqdm(bag.read_messages(), desc="Default rosbag"):
    pass