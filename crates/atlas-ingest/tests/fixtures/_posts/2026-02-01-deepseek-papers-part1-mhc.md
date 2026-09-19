---
layout: post
title: "Deepseek Papers (1/3): mHC - Training Stability at Any Depth"
date: 2026-02-01 00:00:00 -0800
categories: [llm, machine-learning, research]
tags: [deepseek, mhc, transformers, apple-silicon, cuda]
keywords: "training stability, deep networks, initialization, normalization, vanishing gradients"
author: Software Wrighter
abstract: "Implementing Deepseek's mHC (Manifold-Constrained Hyper-Connections) paper. Using Sinkhorn-Knopp iteration to create doubly-stochastic matrices, mHC maintains training stability at 48 layers where standard hyper-connections explode. Cross-platform validation on Apple Silicon and NVIDIA."
series: "Deepseek Papers"
series_part: 1
video_urls:
  - "https://youtube.com/shorts/fh21_zIK2ZE"
  - "https://www.youtube.com/watch?v=MYTXVYDtCEU"
  - "https://youtube.com/shorts/BOuBFn5e1gA"
video_titles:
  - "DeepSeek's mHC Fix for Gradient Explosion"
  - "Deep Networks Fixed by Deepseek mHC"
  - "Sinkhorn on Blackwell (mHC part 2)"
repo_url: "https://github.com/softwarewrighter/mHC-poc"
papers:
  - title: "mHC: Manifold-Constrained Hyper-Connections"
    url: "https://arxiv.org/abs/2512.24880"
---

Body text, which this ingester never reads.
