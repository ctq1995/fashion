declare module '*.svg' {
  const src: string;
  export default src;
}

declare module '*.vue?raw' {
  const src: string;
  export default src;
}

declare module '*?raw' {
  const src: string;
  export default src;
}
