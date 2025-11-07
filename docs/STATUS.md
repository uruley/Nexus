[2025-11-07][Senior Builder] Added runtime patch applier, router schema/examples, perception demo • How to run: `python runtime/main.py` to apply patches, `python perception/demo_depth_pose.py` for demo, `pytest tests/test_runtime_main.py` for verification

[2025-11-07][Senior Builder] Perception service hardening
- [x] Perception server robust when MoveNet/MiDaS absent
- [x] Env-configurable model path (MOVENET_ONNX), camera index
- [x] Downloader utility
- [x] Docs and run scripts added