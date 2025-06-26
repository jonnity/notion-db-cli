# 1.86-bookworm
FROM rust@sha256:251cec8da4689d180f124ef00024c2f83f79d9bf984e43c180a598119e326b84

ARG USERNAME=notion-cli-rs-dev
ARG USER_UID=1001
ARG USER_GID=$USER_UID

# Create the user and add sudo support.
RUN groupadd --gid $USER_GID $USERNAME \
  && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
  && apt-get update \
  && apt-get install -y sudo \
  && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
  && chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME
