from pathlib import Path
import tqdm

import rosbag
from rosbags.highlevel import AnyReader
from rosbags_rs import Bag
from embag import View

# create reader instance and open for reading
with AnyReader([Path("/data/disk0/20210828_heightmaps_1/20210828_12.bag")]) as reader:
    for connection, timestamp, rawdata in tqdm.tqdm(reader.messages(), desc="rosbags"):
        msg = reader.deserialize(rawdata, connection.msgtype)
        # d = msg.fields()

bag = Bag("/data/disk0/20210828_heightmaps_1/20210828_12.bag")
for msg in tqdm.tqdm(bag.read_messages(), desc="rs"):
    pass

view = View()
view.addBag("/data/disk0/20210828_heightmaps_1/20210828_12.bag")

for msg in tqdm.tqdm(view.getMessages(), desc="embag"):
    deser_data = msg.data()
    pass

bag = rosbag.Bag("/data/disk0/20210828_heightmaps_1/20210828_12.bag")
for msg in tqdm.tqdm(bag.read_messages(), desc="Default rosbag"):
    pass