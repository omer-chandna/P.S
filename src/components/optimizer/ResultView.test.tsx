import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ResultView } from "@/components/optimizer/ResultView";

describe("ResultView", () => {
  it("shows an already-clear prompt without a replacement", () => {
    const original = "Add a loading spinner to the Submit button while the existing request is in progress.";
    render(<ResultView status="unchanged" original={original} />);
    expect(screen.getByText("Already clear")).toBeInTheDocument();
    expect(screen.getByText(original)).toBeInTheDocument();
    expect(screen.getByText(/Nothing was pasted/)).toBeInTheDocument();
  });

  it("shows a rewrite beside the original and copies it", async () => {
    const onCopy = vi.fn();
    render(
      <ResultView
        status="rewritten"
        original="fix login it breaks on refresh"
        rewritten="Investigate the login failure after a page refresh."
        warnings={["Missing path: src/auth/session.ts"]}
        onCopy={onCopy}
      />,
    );
    expect(screen.getByText("Rewritten")).toBeInTheDocument();
    expect(screen.getByText("fix login it breaks on refresh")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Copy" }));
    expect(onCopy).toHaveBeenCalledWith("Investigate the login failure after a page refresh.");
    expect(screen.getByText("Missing path: src/auth/session.ts")).toBeInTheDocument();
  });

  it("keeps the original visible when optimization fails", () => {
    render(<ResultView status="error" original="keep me" message="Ollama is not reachable." />);
    expect(screen.getByText("Ollama is not reachable.")).toBeInTheDocument();
    expect(screen.getByText(/left unchanged/)).toBeInTheDocument();
  });
});
