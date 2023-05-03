# OXD File Compression

There should be more files to store in the oxd file than the main XML.
Those files are resource images, text files and etc. So we have to compress
all those files in to a single file and export it as oxd file. We have
couple compression methods.

## Rar

This compression format is not open-source and we can not use it for a open-source
project.

## Zip

Zip is the widely using compression method to compress saved files (Libre Office
and WPS). It also provide a way to store directories and files in the compressed
file. It is faster than the 7z compression algorithm. But the final output is
bigger than the 7z archives. 

## 7z

7z is only a compression method. It is not providing a way to combine set of files
to a one file. But we can use `tar` to do that with 7z. 7z is the smallest compressed
format.

## 7z vs Zip

In our use case we have to reduce the size of the saved files. Because there should be
more asset image in a single project. Since saving and opening is a one-time function,
we do not have to worry about the speed of the compression method. So using the `7z`
and `tar` is ideal for OpenXD.
