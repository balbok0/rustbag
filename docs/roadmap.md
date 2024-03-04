# Roadmap

Following are tasks (in order of priority that we want to focus on):

1. Testing harness - Test harness for Rust side of implementation. This is mostly meant for unit testing, since integration will be done on python side (see below). Also, includes documenting internal crates.
1. Correctness/Reliability - Automated integration test that verify correctness of implementation and returned messages against rosbags
1. Rust crate publishing
1. Python API completeness - Exposing more information to the user. Specifically:
   1. topics - which lists all of topics, with basic metadata
   2. connections - which returns a dictionary from topic to (possibly many) connections
   3. types - which returns a MsgType style tree
1. Optimize performance - The code is inefficient, mostly due to my lack of Rust and PyO3 knowledge. I would like to try to squeeze performance, especially from the Python/Rust interface, which involves a lot of `clone`s.
1. MCap implementation - Extending this library to also read .mcap files. If there are other reasonable ROS2 alternatives I can try to hook into them as well.
1. Msg Index Cache - Caching index of messages (either locally or in a bucket, if write perms allow and user permits) for improved later throughput.
