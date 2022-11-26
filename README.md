# frigg

Download firmware for Samsung devices.

## Installation

```
$ cargo install --git https://github.com/nickelc/frigg.git
```

## Usage

### Check for the latest available firmware
```
$ frigg help check
check for the lastest available firmware version

Usage: frigg check --model <MODEL> --region <REGION>

Options:
  -m, --model <MODEL>    device model
  -r, --region <REGION>  region model
  -h, --help             Print help information
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
download the latest firmware

Usage: frigg download [OPTIONS] --model <MODEL> --region <REGION> [OUTPUT]

Arguments:
  [OUTPUT]  output to a specific file or directory

Options:
  -m, --model <MODEL>    device model
  -r, --region <REGION>  region model
      --download-only    don't decrypt the firmware file
  -h, --help             Print help information
```

### Decrypt a firmware
```
$ frigg help decrypt
decrypt a downloaded firmware

Usage: frigg decrypt --model <MODEL> --region <REGION> --firmware-version <VERSION> <INPUT> [OUTPUT]

Arguments:
  <INPUT>   path to encrypted firmware
  [OUTPUT]  output to a specific file or directory

Options:
  -m, --model <MODEL>               device model
  -r, --region <REGION>             region model
  -v, --firmware-version <VERSION>
  -h, --help                        Print help information
```

#### Example
```
$ frigg decrypt -m GT-I9301I -r DBT -v I9301IXCSAQE1/I9301IDBTAPB1/I9301IXXUAPG1/I9301IXCSAQE1 \
    GT-I9301I_2_20170704182714_xxkuqtgon5_fac1.zip.enc4
```
