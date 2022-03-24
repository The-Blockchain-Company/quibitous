# arguments: quibitous version, target triple, target cpu

import sys
import platform
import hashlib
import shutil
import os


def sha256sum(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        data = f.read()
        h.update(data)
    return h.hexdigest()


version = sys.argv[1]
target = sys.argv[2]
target_cpu = sys.argv[3]

archive_basename = f"quibitous-{version}-{target}-{target_cpu}"

root_dir = f"./target/{target}/release"

if platform.system() == "Windows":
    quibitous_name = "quibitous.exe"
    qcli_name = "qcli.exe"
else:
    quibitous_name = "quibitous"
    qcli_name = "qcli"

quibitous_path = os.path.join(root_dir, quibitous_name)
qcli_path = os.path.join(root_dir, qcli_name)

quibitous_checksum = sha256sum(quibitous_path)
qcli_checksum = sha256sum(qcli_path)

# build archive
if platform.system() == "Windows":
    import zipfile

    content_type = "application/zip"
    archive_name = f"{archive_basename}.zip"
    with zipfile.ZipFile(archive_name, mode="x") as archive:
        archive.write(quibitous_path, arcname=quibitous_name)
        archive.write(qcli_path, arcname=qcli_name)
else:
    import tarfile

    content_type = "application/gzip"
    archive_name = f"{archive_basename}.tar.gz"
    with tarfile.open(archive_name, "x:gz") as archive:
        archive.add(quibitous_path, arcname=quibitous_name)
        archive.add(qcli_path, arcname=qcli_name)

# verify archive
shutil.unpack_archive(archive_name, "./unpack-test")
quibitous1_checksum = sha256sum(os.path.join("./unpack-test", quibitous_name))
qcli1_checksum = sha256sum(os.path.join("./unpack-test", qcli_name))
shutil.rmtree("./unpack-test")
if quibitous1_checksum != quibitous_checksum:
    print(
        f"quibitous checksum mismatch: before {quibitous_checksum} != after {quibitous1_checksum}"
    )
    exit(1)
if qcli1_checksum != qcli_checksum:
    print(f"qcli checksum mismatch: before {qcli_checksum} != after {qcli1_checksum}")
    exit(1)

# save archive checksum
archive_checksum = sha256sum(archive_name)
checksum_filename = f"{archive_name}.sha256"
with open(checksum_filename, "x") as f:
    f.write(archive_checksum)

# set GitHub Action step outputs
print(f"::set-output name=release-archive::{archive_name}")
print(f"::set-output name=release-content-type::{content_type}")
