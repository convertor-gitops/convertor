#!/usr/bin/env just --justfile

install-builder:
    cargo install --path ./crates/builder

version-sync:
    cargo run -p builder -- version sync

dashboard-debug:
    cargo run -p builder -- dashboard debug

# build & push base image
image-base:
    conv image prod --name base -a amd,arm -r crpi-un944o2vo768t7lv.cn-shenzhen.personal.cr.aliyuncs.com --user ""
    conv image prod --name base -a amd,arm -r ghcr.io --user convertor-gitops

image-convd:
    conv image prod --name convd -a amd,arm -r crpi-un944o2vo768t7lv.cn-shenzhen.personal.cr.aliyuncs.com --user ""
    conv image prod --name convd -a amd,arm -r ghcr.io --user convertor-gitops

inspect image:
    docker buildx imagetools inspect {{ image }}
