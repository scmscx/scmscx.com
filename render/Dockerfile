FROM docker.io/ubuntu:25.04 AS build

RUN apt-get update
RUN apt-get install wget lsb-release software-properties-common gnupg -y dialog
RUN wget https://apt.llvm.org/llvm.sh
RUN chmod u+x llvm.sh
RUN ./llvm.sh 19
RUN ln -s /bin/clang-19 /bin/clang
RUN ln -s /bin/clang++-19 /bin/clang++
ENV CC=clang
ENV CXX=clang++
RUN apt-get install cmake curl zip git ninja-build -y dialog
RUN ln -s /usr/bin/ninja /usr/bin/ninja-build
RUN git clone https://github.com/microsoft/vcpkg.git /opt/vcpkg
RUN cd /opt/vcpkg && ./bootstrap-vcpkg.sh -useSystemBinaries && ./vcpkg integrate install
ENV CMAKE_TOOLCHAIN_FILE=/opt/vcpkg/scripts/buildsystems/vcpkg.cmake
ENV VCPKG_ROOT=/opt/vcpkg
# Note that a real linux instance might also want: sudo chown -R $USER:$USER /opt/vcpkg

#Dependencies related to ICU/GLFW/OpenGL
RUN apt-get install pkg-config autoconf automake autoconf-archive -y dialog
RUN apt-get install libxinerama-dev libxcursor-dev xorg-dev libglu1-mesa-dev -y dialog

# Clone and build this project
COPY Chkdraft /Chkdraft

RUN mkdir -p /build
WORKDIR /build
# RUN curl -L -o - https://github.com/scmscx/Chkdraft/archive/1f05cb2f69aeb6e5a2248e344f6fd730a51ec8d5.tar.gz | tar -xz --strip-components=1
RUN cmake -DCMAKE_BUILD_TYPE=Release -G "Ninja" /Chkdraft/src/map_gfx_utils
RUN cmake --build .

FROM ubuntu:25.04 AS run

RUN \
    apt-get update && \
    apt install -y --no-install-recommends \
        xvfb \
        libopengl0 \
        libglu1-mesa \
    && \
    rm -rf /var/lib/apt/lists/*

COPY xvfb-run.sh /usr/bin/xvfb-run.sh

COPY --from=build /build/MapGfxUtils /bin/MapGfxUtils

ENTRYPOINT ["/usr/bin/xvfb-run.sh",  "--auto-servernum", "/bin/MapGfxUtils"]
