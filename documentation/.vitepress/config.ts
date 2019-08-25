import { defineConfig } from 'vitepress'

export default defineConfig({
  title: "Chomsky",
  description: "Universal Grammar and Universal Optimization",
  themeConfig: {
    nav: [
      { text: '首页', link: '/' },
      { text: '普遍语法', link: '/guide/universal-grammar' },
      { text: '普遍优化', link: '/guide/universal-optimization' },
      { text: '输入领域', link: '/domains/' },
      { text: '后端', link: '/backends/' }
    ],
    sidebar: [
      {
        text: '核心理论',
        items: [
          { text: '什么是普遍语法？', link: '/guide/universal-grammar' },
          { text: '什么是普遍优化？', link: '/guide/universal-optimization' },
          { text: '架构分析', link: '/architecture' }
        ]
      },
      {
        text: '输入领域 (Intake)',
        items: [
          { text: '领域概览', link: '/domains/' },
          { text: '编程语言 (nyar-vm)', link: '/domains/programming_languages' },
          { text: '深度学习 (dxo-rs)', link: '/domains/deep_learning' },
          { text: '数据库 (yydb/yyds)', link: '/domains/databases' },
          { text: '分布式调度', link: '/domains/distributed' },
          { text: '硬件综合 (EDA)', link: '/domains/hardware' }
        ]
      },
      {
        text: '硬件后端 (Extraction)',
        items: [
          { text: '总览', link: '/backends/' },
          { text: 'x86_64', link: '/backends/x86_64' },
          { text: 'RISC-V', link: '/backends/riscv' },
          { text: 'CUDA', link: '/backends/cuda' },
          { text: 'SPIR-V', link: '/backends/spirv' },
          { text: 'C', link: '/backends/c' },
          { text: 'JavaScript', link: '/backends/javascript' },
          { text: 'WebAssembly', link: '/backends/wasm' },
          { text: 'JVM', link: '/backends/jvm' },
          { text: 'CLR', link: '/backends/clr' },
          { text: 'eBPF', link: '/backends/ebpf' },
          { text: 'Verilog', link: '/backends/verilog' },
          { text: 'SQL', link: '/backends/sql' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/your-repo/project-chomsky' }
    ]
  }
})
