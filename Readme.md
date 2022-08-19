
## 使用
```bash
ding 0.1.0+master+9a8904d+x86_64-unknown-linux-gnu
jiahong

USAGE:
    prometheus_ding --port <PORT> --ding-url <DING_URL> --title <TITLE>

OPTIONS:
    -d, --ding-url <DING_URL>    
    -h, --help                   Print help information
    -p, --port <PORT>            the web port, eg: 9080
    -t, --title <TITLE>          
    -V, --version                Print version information
```

起服务
```bash
./target/release/prometheus_ding -p 8096 -t test -d https://oapi.dingtalk.com/robot/send?access_token=dd89bd1c283d8aaf3d2d9b7e9d8e6a
```

## 配置
可以在alertmanager的配置文件里这样写
```yml
receivers:
- name: 'web.hook.test'
  webhook_configs:
  - url: 'http://127.0.0.1:8096/ding/markdown' 
```

## 两种报警方式
展示效果
1. 路由`/ding/text`
    ```json
    [
      {
        "status": "firing",
        "labels": {
          "device": "/dev/md0",
          "mountpoint": "/fastdata",
          "fstype": "ext4",
          "job": "deal-node",
          "team": "gongjian",
          "status": "Hight",
          "instance": "172.16.20.52:9100",
          "alertname": "DiskWarn"
        },
        "annotations": {
          "description": "172.16.20.52:9100: deal-node disk need clean!",
          "summary": "172.16.20.52:9100: disk waring!",
          "value": "51.54724021270722"
        },
        "startsAt": "2022-08-19T07:10:11.746Z",
        "endsAt": "0001-01-01T00:00:00Z",
        "generatorURL": "http://527d7c9eba32:9090/graph?g0.expr=%28100+-+%28%28node_filesystem_avail_bytes%7Bjob%3D%22deal-node%22%2Cmountpoint%3D~%22%2Ffastdata%7C%2F%22%7D+%2A+100%29+%2F+node_filesystem_size_bytes%7Bjob%3D%22deal-node%22%2Cmountpoint%3D~%22%2Ffastdata%7C%2F%22%7D%29%29+%3E+50&g0.tab=1",
        "fingerprint": "977519164ae53090"
      }
    ]
    ```

2. 路由`/ding/markdown`
    ```markdown
    ## DiskWarn
    * summary:172.16.20.141:9100: disk waring, used: 51.21347011726213%
    * description:deal-node: 172.16.20.141:9100, the mountpoint: /fastdata disk need clean!
    ## DiskWarn
    * summary:172.16.20.52:9100: disk waring, used: 55.63773960426617%
    * description:deal-node: 172.16.20.52:9100, the mountpoint: /fastdata disk need clean!
    ```

## 如果想看更多日志或者调试
使用RUST_LOG=debug，环境变量
