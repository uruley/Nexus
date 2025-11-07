# MoveNet model

This directory should contain the `movenet_thunder.onnx` model file. The
server expects the file at `perception/models/movenet_thunder.onnx`.

Because the automated build environment used for this commit cannot access the
internet, the model weights are not included. To run the server locally, place a
copy of the MoveNet Thunder ONNX export in this directory before starting the
FastAPI app.
