# frigg

Download firmware for Samsung devices.

## Installation

```
$ cargo install https://github.com/nickelc/frigg.git
```

## Usage

### Check for the latest available firmware
```
$ frigg help check
frigg-check
check for the lastest available firmware version

USAGE:
    frigg check --model <MODEL> --region <REGION>

FLAGS:
    -h, --help    Prints help information

OPTIONS:
    -m, --model <MODEL>      device model
    -r, --region <REGION>    device region

```

#### Example
```
$ frigg check --model GT-I9301I --region DBT
Name: GALAXY S Ⅲ Neo
Model: GT-I9301I
Region: DBT
Latest Version:
  Version: I9301IXCSAQE1/I9301IDBTAPB1/I9301IXXUAPG1/I9301IXCSAQE1
  Filename: GT-I9301I_2_20170704182714_xxkuqtgon5_fac.zip.enc4
  Size: 1178062496 bytes
  Decrypt key: 824ED914CCA75970EBDFC07132C23E09
```

### Download a firmware
```
$ frigg help download
frigg-download
download the latest firmware

USAGE:
    frigg download [FLAGS] --model <MODEL> --region <REGION> [OUTPUT]

FLAGS:
        --download-only    don't decrypt
    -h, --help             Prints help information

OPTIONS:
    -m, --model <MODEL>      device model
    -r, --region <REGION>    device region

ARGS:
    <OUTPUT>    output to a specific file or directory

```

### Decrypt a firmware
```
$ frigg help decrypt
frigg-decrypt
decrypt a downloaded firmware

USAGE:
    frigg decrypt <INPUT> --model <MODEL> --region <REGION> --firmware-version <VERSION> [OUTPUT]

FLAGS:
    -h, --help    Prints help information

OPTIONS:
    -m, --model <MODEL>                 device model
    -r, --region <REGION>               device region
    -v, --firmware-version <VERSION>

ARGS:
    <INPUT>     path to encrypted firmware
    <OUTPUT>    output to a specific file or directory
```

#### Example
```
$ frigg decrypt -m GT-I9301I -r DBT -v I9301IXCSAQE1/I9301IDBTAPB1/I9301IXXUAPG1/I9301IXCSAQE1 \
    GT-I9301I_2_20170704182714_xxkuqtgon5_fac1.zip.enc4
```
