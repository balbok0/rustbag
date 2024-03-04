# Performance

To provide transparency, here are benchmarks for rustbag and alternatives. Since rustbag is still in alpha, without fully developed feature set (check [roadmap](./roadmap.md)), there will not be a lot of focus put into improving these values in short term. However, in-long term, I hope this page to provide in-depth understanding of pros and cons of each of the rosbag/mcap readers.

## Local machine

### Rellis-3D (20210828_2.bag; 2.9GB)

| library | time taken | iterations/second |
| --- | --- | --- |
| rustbag | 00:07 | 2514.72 |
| rosbags | 00:08 | 2239.63 |
| embag | 00:07 | 2318.10 |
| rosbag | 00:09 | 1836.96 |

### Rellis-3D (20210828_15.bag; 17GB)

| library | time taken | iterations/second |
| --- | --- | --- |
| rustbag | 00:50 | 2055.44 |
| rosbags | 00:58 | 1750.83 |
| embag | 01:01 | 1676.42 |
| rosbag | 01:43 | 989.70 |

### Udacity (HMB_1.bag; 672MB)

| library | time taken | iterations/second |
| --- | --- | --- |
| rustbag | 00:07 | 116281.31 |
| rosbags | 00:17 | 51035.87 |
| embag | 00:02 | 366963.75 |
| rosbag | 00:20 | 45357.38 |

## Bag from docker container on same machine

### Rellis-3D (20210828_2.bag; 2.9GB)

| library | time taken | iterations/second |
| --- | --- | --- |
| rustbag | 00:08 | 2214.61 |
| rosbags | 00:43 | 417.09 |
| embag | 00:48 | 375.47 |
| rosbag | 00:29 | 627.79 |

### Udacity (HMB_1.bag; 672MB)

| library | time taken | iterations/second |
| --- | --- | --- |
| rustbag | 00:11 | 80168.29 |
| rosbags | 00:18 | 49279.25 |
| embag | 00:04 | 209895.66 |
| rosbag | 00:21 | 42965.35 |
