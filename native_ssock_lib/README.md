# tassl_wrap
用cpp和c语言包装国密的tls/ssl实现，包括cpp动态库，c语言native api接口库，测试代码等

根据传入的证书不同，可以同时支持国密和非国密。

### 总体结构说明

最底层依赖TASSL库，用c风格在不同操作系统平台上封装，代码差异主要在对socket编程上，linux平台和windows平台的socket编程略有不同。

本代码在bcos的python sdk里先实现适配了，考虑到可以复用，则独立为一个项目。

在windows上，因开发环境使用的是msys2+mingw64,和msvc编译环境并不兼容



### 目录结构

**cpp_linux**: cpp封装代码，linux centos7平台支持

**cpp_win**: cpp封装代码，windows平台支持，在msys2+mingw64上编译

**cpp_common**: 操作系统平台相关性不那么强的一些公共代码

----------------------------------

### 准备TASSL环境

1. 获取TASSL代码：git clone https://github.com/jntass/TASSL (或gitee镜像)

2. 进入TASSL目录，config 

> 注意用shared选项生成动态库
> 
> ./config shared
>   
>make clean ；make ；make install

此刻应在TASSL目录下生成libssl.so,libcrypto.so 等文件。

* 在window上，则是生成 ssleay32.dll，libeay32.dll等文件

(todo: TASSL是否应编译成特定的名字，如libtassl.so,目前是默认的libssl.so)

是否安装TASSL的库到系统路径，取决于实际环境和需求。

3. 编辑环境变量，如~/.bash_profile,增加 

```bash
export TASSL=[实际的TASSL目录，需要检索头文件，lib库等]

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:./:$TASSL

export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:[能找到native_tassl_sock_wrap.so/dll运行依赖库的目录]


```
并确认环境变量生效



--------------------------------------------

### msys2环境搭建
在windows环境上，MSYS2是MSYS的一个升级版,集成了pacman和Mingw-w64的Cygwin升级版, 
提供了bash shell等linux环境（仿真）、版本控制软件（git/hg）和MinGW-w64 工具链（来自[百度百科](https://baike.baidu.com/item/MSYS2/17190550))

官网:[https://www.msys2.org/](https://www.msys2.org/)

配置加速镜像：

> 编辑 /etc/pacman.d/mirrorlist.mingw64 ，在文件开头添加：
> 
>Server = https://mirrors.tuna.tsinghua.edu.cn/msys2/mingw/x86_64
>
>编辑 /etc/pacman.d/mirrorlist.msys ，在文件开头添加：
>
>Server = https://mirrors.tuna.tsinghua.edu.cn/msys2/msys/$arch
> 
>然后执行 pacman -Sy 刷新软件包数据即可。
>
>pacman -S mingw-w64-x86_64-gcc mingw-w64-x86_64-cmake mingw-w64-x86_64-make mingw-w64-x86_64-pkg-config 
>
>升级核心包: pacman -S --needed filesystem msys2-runtime bash libreadline libiconv libarchive libgpgme libcurl pacman ncurses libintl, 之后需要关闭所有 MSYS2 shell，然后运行 autorebase.bat

* 谨建议对linux风格有偏好的使用，考虑兼容和稳定性，建议安装配置MSVC开发环境，并将cpp_win下的各代码加入到MSVC的项目，参考Makefile进行编译设置。
* 本项目全程在msys+mingw64下编译开发，暂时不另外提供MSVC工程，欢迎开源贡献.

