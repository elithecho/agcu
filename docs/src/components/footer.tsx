export function Footer() {
  return (
    <footer className="mt-auto">
      <div className="mx-auto max-w-[72rem] px-6 py-6">
        <div className="flex items-center justify-between text-[11px] text-muted-foreground/40">
          <div className="flex items-center gap-2">
            <span>agcu — Agent Computer Use</span>
            <span>&middot;</span>
            <span>MIT</span>
          </div>
          <div className="flex items-center gap-3">
            <a
              href="https://github.com/elithecho/agcu"
              className="hover:text-muted-foreground transition-colors"
            >
              GitHub
            </a>
            <a href="/changelog" className="hover:text-muted-foreground transition-colors">
              Changelog
            </a>
            <a
              href="https://www.npmjs.com/package/agcu"
              className="hover:text-muted-foreground transition-colors"
            >
              npm
            </a>
          </div>
        </div>
      </div>
    </footer>
  );
}
