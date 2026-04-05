import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { useState } from "react";
import userEvent from "@testing-library/user-event";
import { Textarea } from "./textarea";

// Note: happy-dom doesn't track scrollHeight for auto-resize, so we test
// callback behaviour rather than visual height changes.

describe("Textarea", () => {
  it("renders with label", () => {
    render(<Textarea value="hello" label="Notes" lines={3} />);
    expect(screen.getByText("Notes")).toBeInTheDocument();
  });

  it("renders with the given value", () => {
    render(<Textarea value="my text" label="Label" lines={2} />);
    expect(screen.getByRole("textbox")).toHaveValue("my text");
  });

  it("uses the same hover background class as default text inputs", () => {
    render(<Textarea value="my text" label="Label" lines={2} />);
    expect(screen.getByRole("textbox").className).toContain(
      "hover:bg-neutral-700",
    );
  });

  it("calls onChange when user types", () => {
    const onChange = vi.fn();
    render(<Textarea value="" label="Label" onChange={onChange} lines={1} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.input(textarea, { target: { value: "abc" } });
    expect(onChange).toHaveBeenCalledWith("abc");
  });

  it("does not throw when onChange is not provided", () => {
    render(<Textarea value="" label="Label" lines={1} />);
    const textarea = screen.getByRole("textbox");
    expect(() =>
      fireEvent.input(textarea, { target: { value: "x" } }),
    ).not.toThrow();
  });

  it("label turns sky-500 on focus", () => {
    render(<Textarea value="" label="Notes" lines={2} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.focus(textarea);
    expect(screen.getByText("Notes").className).toContain("text-sky-500");
  });

  it("label reverts on blur", () => {
    render(<Textarea value="" label="Notes" lines={2} />);
    const textarea = screen.getByRole("textbox");
    fireEvent.focus(textarea);
    fireEvent.blur(textarea);
    expect(screen.getByText("Notes").className).toContain("text-neutral-400");
  });

  it("sets contentEditable=false on textarea when readOnly=true", () => {
    render(<Textarea value="ro" label="Label" lines={1} readOnly />);
    expect(screen.getByRole("textbox")).toHaveAttribute(
      "contenteditable",
      "false",
    );
  });

  it("shows local draft text while focused", async () => {
    const ControlledTextarea = () => {
      const [value, setValue] = useState("hello");

      return (
        <Textarea value={value} label="Notes" lines={2} onChange={setValue} />
      );
    };

    const user = userEvent.setup();
    render(<ControlledTextarea />);

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    await user.click(textarea);
    await user.type(textarea, "X");

    expect(textarea).toHaveValue("helloX");
  });

  it("keeps the local draft visible while async parent state lags", async () => {
    const AsyncControlledTextarea = () => {
      const [value, setValue] = useState("hello");

      return (
        <Textarea
          value={value}
          label="Notes"
          lines={2}
          onChange={(nextValue) => {
            setTimeout(() => setValue(nextValue), 0);
          }}
        />
      );
    };

    const user = userEvent.setup();
    render(<AsyncControlledTextarea />);

    const textarea = screen.getByRole("textbox") as HTMLTextAreaElement;
    await user.click(textarea);
    await user.type(textarea, "X");

    expect(textarea).toHaveValue("helloX");
  });
});
