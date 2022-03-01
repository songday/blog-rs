## 如何使用

### 1、启动本工具
可以通过命令行来设置，执行：`blog-backend.exe -h`可以看到帮助信息
```
USAGE:
    blog-backend.exe [OPTIONS]

OPTIONS:
        --cert-path <CERT_PATH>      Cert file path, needed by https
        --cors-host <CORS_HOST>      Hostname for CORS
    -h, --help                       Print help information
        --hsts-enabled               Enable HSTS Redirect Server
        --https-enabled              Enable HTTPS Server
        --https-port <HTTPS_PORT>    Specify HTTPS listening port, default value is '443' [default:
                                     443]
        --ip <IP>                    HTTP Server Settings Specify http listening address, e.g.:
                                     0.0.0.0 or [::] or 127.0.0.1 or other particular ip, default is
                                     '127.0.0.1' [default: 127.0.0.1]
        --key-path <KEY_PATH>        Key file path, needed by https
        --mode <MODE>                Specify run mode: 'static' is for static file serve, 'blog' is
                                     blog warp server mode
        --port <PORT>                Specify listening port, default value is '80' [default: 80]
    -V, --version                    Print version information
```

根据上面的信息，可以了解到，直接执行：`blog-backend.exe`，该服务会启动`HTTP`服务，默认监听：`127.0.0.1:80`  
访问：[http://localhost](http://localhost) 即可

如果要修改端口，可以使用：`--port`参数。如：`blog-backend.exe --port 9270`  
然后访问：[http://localhost:9270](http://localhost:9270) 即可

### 2、设置管理员密码
在没有设置管理员密码的时候，系统会自动打开如下页面。  
输入密码（最少1位），点击：“更新”即可

## 如何将我的博客展现给其他人看？

### 1、使用本工具自带的HTTP服务器
在上面的“启动本工具”环境，介绍了如何启动。   
我们仅需要做一些小调整，就可以对外服务了。  
1. 使用`--ip`，修改为：`0.0.0.0`，或其它外网IP
2. `--https-enabled`是用于启用`HTTPS`（需配合`--cert-path`、`--key-path`参数）

### 2、导出到Hugo服务器
在`管理`页面，可以导出为`Hugo`静态文件，使用`Hugo`来渲染。

### 3、使用本工具的静态文件服务模式
启动的时候，指定`--mode static`即可使用该模式。

