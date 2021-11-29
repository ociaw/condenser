Condenser
---------

Condenser is a CLI application designed to merge and transform the contents of
one or more input diriectories into a single output directory.

For example, this is useful when you have a large, diverse music archive - there's a lot
of high quality, high bitrate (perhaps lossless) music in there from different sources
across different genres. But that won't all fit on your phone, so you need a subset of
the archive, and maybe compress it too. This is where `Condenser` steps in: it allows you
to pick and choose which music you want with `glob` and `regex` filters, while also
letting you compress it, say from `FLAC` to `opus`.

File Discovery
--------------

Each input folder is scanned independently of others and matched against a glob.
Files not matching the glob are ignored.
Each input folder is represented by an object that contains the *relative* paths of all files.
Each file path is relative to the input folder.
Each file path is unique within the input folder.
File paths are not necessarily unique between input folders.

Transfomers
----------

Output transfomers are arranged by priority - higher priority transfomers win file path conflicts.

Output Path Mapping
-------------------

The first step in output processing is matching transformers to input files. Each input directory
is scanned and filtered. Then each transformer runs in order of priority, claiming and removing
matching files that have not yet been claimed. Whenever a tranformer finds a matching file, it
claims the file - i.e., this input file cannot be claimed by other transformers. Additionally,
the transformer claims the output file id - so no other file can be output that results in the
same file. This process repeats for each input folder, in order of the input directory priority.

Orphan Deletion
---------------
The output directory is then scanned, and any files that are not in the claimed output paths are
deleted.

Output Transformation
-----------------

Now that all inputs have been mapped to outputs and to transfomers, each transfomer is called.
Since output filenames can't be shared between transfomers, and inputs are read-only, this can
be done in parallel (though currently this is not the case).

An Example
----------

An example use case is transforming an archive library of high-quality audio to lower quality audio,
such as for use on a device with limited storage like a mobile phone. There are multiple formats
stored within the archive, including lossless FLAC and WAV files, as well as high- and low-bitrate
MP3 files (320 kbps and 128kbps). Two input directories exist, one containing cd-extracted audio and
the other containing downloaded audio. Since there are two input directories, duplicates can exist,
such as an album that has been both downloaded online and extracted from a physical cd. Different
bitrates can also exist - the CD extracted audio may be lossless, and the downloaded can be lossily
compressed by MP3.

The audio in the output library should be compressed to 160kbps AAC, *unless* the source is of lower
bitrate already, in which case it is passed through.

Two transformers will used:
1. a "Transcode to AAC" transformer with a priority of 100
2. a "Copy" transformer with a priority of 50

Transformer 1 will transcode audio to AAC if the bitrate is higher than 160kbps, skipping them otherwise.
Transformer 2 will simply copy all files over.

Due to the priorities, Tranformer 2 will only be invoked for outputs that Transformer 1 hasn't
handled.
