FROM ubuntu:jammy

ENV LC_ALL en_US.UTF-8
ENV KURP_CONF_DIR="/config"

WORKDIR app
RUN apt-get update && apt-get -y install locales  \
    && locale-gen en_US.UTF-8 \
    && apt-get -y install wget unzip git \
    && apt-get -y install libvulkan1 libgomp1 \
    && wget https://github.com/Tencent/ncnn/releases/download/20230223/ncnn-20230223-ubuntu-2204-shared.zip -O ncnn.zip \
    && unzip ncnn.zip \
    && mv ./ncnn-20230223-ubuntu-2204-shared/lib/libncnn.so.1.0.20230223 /usr/lib \
    && ln -s /usr/lib/libncnn.so.1.0.20230223 /usr/lib/libncnn.so \
    && ln -s /usr/lib/libncnn.so.1.0.20230223 /usr/lib/libncnn.so.1 \
    && rm -rf ncnn-20230223-ubuntu-2204-shared.zip \
    && rm -rf ncnn.zip \
    && git clone https://github.com/nihui/waifu2x-ncnn-vulkan \
    && mv waifu2x-ncnn-vulkan/models . \
    && git clone https://github.com/nihui/realcugan-ncnn-vulkan \
    && mv realcugan-ncnn-vulkan/models/* ./models \
    && rm -rf waifu2x-ncnn-vulkan \
    && rm -rf realcugan-ncnn-vulkan \
    && apt-get -y remove wget unzip git \
    && apt-get -y autoremove \
    && apt-get clean

COPY target/release/kurp ./

ENTRYPOINT ["./kurp"]
EXPOSE 3030
