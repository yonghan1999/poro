### windows管理员权限启动
打包之后，使用下面的命令
~~~cmd
mt.exe -manifest "filename.manifest" -outputresource:"example.exe";#1
~~~
例如：在我本机上需要执行
~~~cmd
mt.exe -manifest "F:\github\poro\src\windows_program.manifest" -outputresource:"F:\github\poro\target\release\poro.exe";#1
~~~
