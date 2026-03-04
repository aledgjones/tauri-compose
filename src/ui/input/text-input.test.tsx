import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TextInput } from "./text-input";

describe("TextInput", () => {
  it("renders with initial value", () => {
    render(<TextInput type="text" value="hello" onChange={vi.fn()} />);
    expect(screen.getByRole("textbox")).toHaveValue("hello");
  });

  it("calls onChange when user types", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();
    render(<TextInput type="text" value="" onChange={onChange} />);
    await user.type(screen.getByRole("textbox"), "A");
    expect(onChange).toHaveBeenCalled();
  });

  it("renders label when provided", () => {
    render(<TextInput type="text" value="" onChange={vi.fn()} label="Name" />);
    expect(screen.getByText("Name")).toBeInTheDocument();
  });

  it("label turns sky-500 on focus", async () => {
    const user = userEvent.setup();
    render(<TextInput type="text" value="" onChange={vi.fn()} label="Name" />);
    const input = screen.getByRole("textbox");
    await user.click(input);
    const label = screen.getByText("Name");
    expect(label.className).toContain("text-sky-500");
  });

  it("applies readOnly to the input element", () => {
    render(<TextInput type="text" value="ro" onChange={vi.fn()} readOnly />);
    expect(screen.getByRole("textbox")).toHaveAttribute("readonly");
  });

  it("fires onEnter when Enter key is pressed", async () => {
    const onEnter = vi.fn();
    const user = userEvent.setup();
    render(
      <TextInput type="text" value="" onChange={vi.fn()} onEnter={onEnter} />,
    );
    await user.type(screen.getByRole("textbox"), "{Enter}");
    expect(onEnter).toHaveBeenCalledOnce();
  });

  it("fires onBlur when input loses focus", () => {
    const onBlur = vi.fn();
    render(
      <TextInput type="text" value="" onChange={vi.fn()} onBlur={onBlur} />,
    );
    const input = screen.getByRole("textbox");
    fireEvent.focus(input);
    fireEvent.blur(input);
    expect(onBlur).toHaveBeenCalledOnce();
  });

  it("renders units suffix when provided", () => {
    render(<TextInput type="text" value="10" onChange={vi.fn()} units="px" />);
    expect(screen.getByText("px")).toBeInTheDocument();
  });

  it("applies dark variant bg class", () => {
    render(
      <TextInput type="text" value="" onChange={vi.fn()} variant="dark" />,
    );
    expect(screen.getByRole("textbox").className).toContain("bg-neutral-900");
  });

  it("applies default variant bg class", () => {
    render(
      <TextInput type="text" value="" onChange={vi.fn()} variant="default" />,
    );
    expect(screen.getByRole("textbox").className).toContain("bg-neutral-800");
  });
});
