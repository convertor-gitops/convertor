#!/usr/bin/env node

/**
 * 从 metadata.json 生成 TypeScript 文件
 */

const fs = require('fs');
const path = require('path');

const rootDir = path.resolve(__dirname, '../..');
const metadataPath = path.join(rootDir, 'metadata.json');
const outputPath = path.join(__dirname, '../src/metadata.ts');

try {
  // 读取 metadata.json
  const metadata = JSON.parse(fs.readFileSync(metadataPath, 'utf-8'));

  // 生成 TypeScript 文件
  const tsContent = `/**
 * 项目元数据
 * 此文件由构建脚本自动生成，请勿手动修改
 * 数据源: ../../metadata.json
 */

export interface Metadata {
  name: string;
  repository: string;
  description: string;
  author: string;
  license: string;
  version: string;
  build: number;
}

export const metadata: Metadata = ${JSON.stringify(metadata, null, 2)};

export const { name, repository, description, author, license, version, build } = metadata;
`;

  fs.writeFileSync(outputPath, tsContent, 'utf-8');
  console.log('✅ 已生成 src/metadata.ts');
} catch (error) {
  console.error('❌ 生成失败:', error.message);
  process.exit(1);
}

