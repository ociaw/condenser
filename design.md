Mass File Transmuter
--------------------

Transformer and Processor are equivalent.

A library is defined by its output folder, and is fed by zero or more input folders.

File Discovery
--------------

Each input folder is scanned independently of others and matched against a glob.
Files not matching the glob are ignored.
Each input folder is represented by an object that contains the *relative* paths of all files.
Each file path is relative to the input folder.
Each file path is unique within the input folder.
File paths are not necessarily unique between input folders.

Processors
----------

Output processors are arranged by priority - higher priority processors win file path conflicts.

Output Path Mapping
-------------------

The first step in output processing is determining the output file names. Every found file
is passed to each processor. The processor examines the file and either returns a `file path id`
(typically just the file path without an extension, relative to the output directory), or returns
None if it is unable to process the file.

Once all processors have completed path mapping, the `file path ids` are checked for uniqueness.
In the case that multiple processors produce the same `file path ids`, the processor with the
higher priority is chosen for that `file path id`.

Output Processing
-----------------

Now that all inputs have been mapped to outputs and to processors, each processor is called.
Since output filenames can't be shared between processors, and inputs are read-only, this can
be done in parallel.

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

