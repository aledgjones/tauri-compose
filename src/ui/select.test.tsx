import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Select } from "./select";
import type { SelectOption } from "./select";

type Letter = "a" | "b" | "c";

const OPTIONS: SelectOption<Letter>[] = [
  { value: "a", label: "Apple" },
  { value: "b", label: "Banana" },
  { value: "c", label: "Cherry" },
];

const OPTIONS_WITH_DISABLED: SelectOption<Letter>[] = [
  { value: "a", label: "Apple" },
  { value: "b", label: "Banana", disabled: true },
  { value: "c", label: "Cherry" },
];

const OPTIONS_WITH_END_ADORNMENT: SelectOption<Letter>[] = [
  { value: "a", label: "Apple" },
  {
    value: "b",
    label: "Banana",
    disabled: true,
    endAdornment: <span title="Needs attention">!</span>,
  },
  { value: "c", label: "Cherry" },
];

describe("Select", () => {
  it("renders with placeholder when value is null", () => {
    render(
      <Select
        value={null}
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
        placeholder="Pick a fruit"
      />,
    );
    expect(screen.getByText("Pick a fruit")).toBeInTheDocument();
  });

  it("shows current selection label", () => {
    render(
      <Select
        value="b"
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
      />,
    );
    expect(screen.getByText("Banana")).toBeInTheDocument();
  });

  it("dropdown is closed by default — no options visible", () => {
    render(
      <Select
        value="a"
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
      />,
    );
    expect(screen.queryByRole("option")).not.toBeInTheDocument();
  });

  it("clicking the button opens the dropdown and shows all options", async () => {
    const user = userEvent.setup();
    render(
      <Select
        value="a"
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
      />,
    );
    const button = screen.getByRole("button");
    await user.click(button);
    expect(screen.getAllByRole("option")).toHaveLength(3);
    expect(screen.getByRole("option", { name: "Apple" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Banana" })).toBeInTheDocument();
    expect(screen.getByRole("option", { name: "Cherry" })).toBeInTheDocument();
  });

  it("selecting an option calls onChange with the correct value", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <Select
        value="a"
        options={OPTIONS}
        onChange={onChange}
        variant="standard"
      />,
    );
    await user.click(screen.getByRole("button"));
    await user.click(screen.getByText("Cherry"));
    expect(onChange).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalledWith("c");
  });

  it("selecting an option closes the dropdown", async () => {
    const user = userEvent.setup();
    render(
      <Select
        value="a"
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
      />,
    );
    await user.click(screen.getByRole("button"));
    expect(screen.getAllByRole("option")).toHaveLength(3);
    await user.click(screen.getByText("Banana"));
    expect(screen.queryByRole("option")).not.toBeInTheDocument();
  });

  it("clicking outside closes the dropdown", async () => {
    const user = userEvent.setup();
    render(
      <div>
        <div data-testid="outside">Outside</div>
        <Select
          value="a"
          options={OPTIONS}
          onChange={vi.fn()}
          variant="standard"
        />
      </div>,
    );
    await user.click(screen.getByRole("button"));
    expect(screen.getAllByRole("option")).toHaveLength(3);
    fireEvent.mouseDown(screen.getByTestId("outside"));
    expect(screen.queryByRole("option")).not.toBeInTheDocument();
  });

  it("disabled prop prevents dropdown from opening", async () => {
    const user = userEvent.setup();
    render(
      <Select
        value="a"
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
        disabled
      />,
    );
    await user.click(screen.getByRole("button"));
    expect(screen.queryByRole("option")).not.toBeInTheDocument();
  });

  it("renders a non-interactive end adornment inside the select button", () => {
    render(
      <Select
        value="a"
        options={OPTIONS}
        onChange={vi.fn()}
        variant="standard"
        endAdornment={<span title="Needs attention">!</span>}
      />,
    );

    const button = screen.getByRole("button", { name: /apple/i });

    expect(button).toContainElement(screen.getByTitle("Needs attention"));
  });

  it("marks disabled options as unavailable in the open list", async () => {
    const user = userEvent.setup();
    render(
      <Select
        value="a"
        options={OPTIONS_WITH_DISABLED}
        onChange={vi.fn()}
        variant="standard"
      />,
    );

    await user.click(screen.getByRole("button", { name: /apple/i }));

    expect(screen.getByRole("option", { name: "Banana" })).toBeDisabled();
  });

  it("does not call onChange when a disabled option is clicked", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <Select
        value="a"
        options={OPTIONS_WITH_DISABLED}
        onChange={onChange}
        variant="standard"
      />,
    );

    await user.click(screen.getByRole("button", { name: /apple/i }));
    await user.click(screen.getByRole("option", { name: "Banana" }));

    expect(onChange).not.toHaveBeenCalled();
    expect(screen.getByRole("option", { name: "Banana" })).toBeInTheDocument();
  });

  it("renders option adornments inside the open list", async () => {
    const user = userEvent.setup();

    render(
      <Select
        value="a"
        options={OPTIONS_WITH_END_ADORNMENT}
        onChange={vi.fn()}
        variant="standard"
      />,
    );

    await user.click(screen.getByRole("button", { name: /apple/i }));

    const bananaOption = screen.getByRole("option", { name: /banana/i });

    expect(bananaOption).toContainElement(screen.getByTitle("Needs attention"));
  });
});
