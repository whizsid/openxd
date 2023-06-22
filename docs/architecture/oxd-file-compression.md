# OXD File Compression

There should be more files to store in the oxd file than the main XML.
Those files are resource images, text files and etc. So we have to compress
all those files in to a single file and export it as oxd file. We have
couple compression methods.

## Rar

We have to avoid using RAR format due to open source licensing issues.

## Zip

Zip is the widely using compression method to compress saved files (Libre Office
and WPS). It also provide a way to store directories and files in the compressed
file. Zip compression is faster than other algorithms. But the final output is
bigger. But we can reduce the file size by using a appropriate compression algorithm.
We referred [this](https://linuxreviews.org/Comparison_of_Compression_Algorithms)
comparison and found that the xz compression algorithm producing a small output.

## Tar with compression algorithm

Tar is only a archive method and we have to use it with a compression algorithm to
get a minimal output size. We tried tar with xz compression algorithm and found that
this approach is not working for cross-software uses. Because we have to maintain a
standard for compression options. If a software decompressed a file compressed
with another software using different options, then the decompression will fail.
