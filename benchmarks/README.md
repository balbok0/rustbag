# Benchmark suite

## First Time Setup
You will need [Miniforge](https://github.com/conda-forge/miniforge?tab=readme-ov-file#download).
Then run `mamba create -f ./environment.yaml` in this directory.
To activate created environment run `mamba activate benches-rustbag` (or `conda activate benches-rustbag`).
Further guide assumes that you are in this environment (i.e. you have activated it).

### "Remote" storage setup
We will simulate remote storage using a test deployment of MinIO.
You will need [Docker](https://docs.docker.com/engine/install/) and [Docker Compose](https://docs.docker.com/compose/install/).

Then run `docker compose up`.
If this is your first time starting up MinIO for benchmarks, you will need to navigate to *http://localhost:9000*.
```
Login: minioadmin
Password: minioadmin
```

Then:

1. Create a bucket named: test-bags
2. Upload bags you want to use into that bucket.

If you are planning on testing remote reads with libraries other than rustbag, you will also need to:

1. Install [s3fs](https://github.com/s3fs-fuse/s3fs-fuse)
1. Create a file called `.minio-creds` in root of this repository (important: you will need to `chmod 600 .minio-creds`). It's contents should be `minioadmin:minioadmin`
1. Create a folder called `s3` in root of this repository
2. Run `s3fs <bucket-name> ./s3 -o passwd_file=.minio-creds,use_path
_request_style,url=http://localhost:9000` in the root of the repo

## Running code

### Testing local bags
TODO: Modify script to use parse_args
`python check_rosbags.py`

## Useful bag sources
Try to use bags as close to ones you will be using. Otherwise, here are some useful sources in no particular order. Please note that some of these sources are under more restrictive licenses than rustbag, and are **not** ok for commercial use.

* [Rellis-3D](https://github.com/unmannedlab/RELLIS-3D) - Various sequences of data with bag sizes of 2GB up to ~40GB
* [Tartan Drive](https://github.com/castacks/tartan_drive) - Around 15 sequences (bags) of data as a single 100GB zip file. Sizes range from <1Gb to 15Gb.
* [Kitti](https://www.cvlibs.net/datasets/kitti/raw_data.php) + [kitti2bag](https://github.com/tomas789/kitti2bag)
* [Waymo Open Dataset](https://waymo.com/open/) + [waymo2bag](https://github.com/yukke42/waymo2bag)
