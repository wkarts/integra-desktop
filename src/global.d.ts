declare module '*.html?url' {
  const src: string;
  export default src;
}

declare namespace React {
  interface InputHTMLAttributes<T> {
    webkitdirectory?: string;
    directory?: string;
  }
}
