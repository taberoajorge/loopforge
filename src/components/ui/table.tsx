import { cva, type VariantProps } from "class-variance-authority";
import * as React from "react";

import { cn } from "@/lib/utils";

const tableContainerVariants = cva(
  "min-h-0 min-w-0 overflow-auto rounded-lg border border-border bg-surface",
);

const tableRowVariants = cva(
  "border-border/50 border-b text-text transition-colors last:border-0",
  {
    variants: {
      interactive: {
        true: "hover:bg-elevated/50",
        false: "",
      },
      state: {
        default: "",
        active: "border-primary/30 bg-primary/10",
        muted: "text-text-muted",
        selected: "bg-elevated/80",
      },
    },
    defaultVariants: {
      interactive: true,
      state: "default",
    },
  },
);

export type TableContainerProps = React.HTMLAttributes<HTMLDivElement>;
export type TableProps = React.TableHTMLAttributes<HTMLTableElement>;

const TableContainer = React.forwardRef<HTMLDivElement, TableContainerProps>(
  ({ className, ...props }, ref) => {
    return <div ref={ref} className={cn(tableContainerVariants(), className)} {...props} />;
  },
);

TableContainer.displayName = "TableContainer";

const Table = React.forwardRef<HTMLTableElement, TableProps>(({ className, ...props }, ref) => {
  return (
    <table
      ref={ref}
      className={cn("w-full caption-bottom border-separate border-spacing-0 text-sm", className)}
      {...props}
    />
  );
});

Table.displayName = "Table";

export type TableHeaderProps = React.HTMLAttributes<HTMLTableSectionElement>;

const TableHeader = React.forwardRef<HTMLTableSectionElement, TableHeaderProps>(
  ({ className, ...props }, ref) => {
    return <thead ref={ref} className={cn("bg-surface", className)} {...props} />;
  },
);

TableHeader.displayName = "TableHeader";

export type TableBodyProps = React.HTMLAttributes<HTMLTableSectionElement>;

const TableBody = React.forwardRef<HTMLTableSectionElement, TableBodyProps>(
  ({ className, ...props }, ref) => {
    return <tbody ref={ref} className={cn("[&_tr:last-child]:border-0", className)} {...props} />;
  },
);

TableBody.displayName = "TableBody";

export type TableFooterProps = React.HTMLAttributes<HTMLTableSectionElement>;

const TableFooter = React.forwardRef<HTMLTableSectionElement, TableFooterProps>(
  ({ className, ...props }, ref) => {
    return (
      <tfoot
        ref={ref}
        className={cn("border-border border-t bg-elevated/60 font-medium text-text", className)}
        {...props}
      />
    );
  },
);

TableFooter.displayName = "TableFooter";

export type TableRowProps = React.HTMLAttributes<HTMLTableRowElement> &
  VariantProps<typeof tableRowVariants>;

const TableRow = React.forwardRef<HTMLTableRowElement, TableRowProps>(
  ({ className, interactive, state, ...props }, ref) => {
    return (
      <tr
        ref={ref}
        className={cn(tableRowVariants({ interactive, state }), className)}
        data-state={state ?? "default"}
        {...props}
      />
    );
  },
);

TableRow.displayName = "TableRow";

export type TableHeadProps = React.ThHTMLAttributes<HTMLTableCellElement> & {
  numeric?: boolean;
};

const TableHead = React.forwardRef<HTMLTableCellElement, TableHeadProps>(
  ({ className, numeric = false, ...props }, ref) => {
    return (
      <th
        ref={ref}
        className={cn(
          "h-11 px-4 align-middle font-sans text-text-muted text-xs uppercase tracking-wider",
          numeric ? "text-right tabular-nums" : "text-left",
          className,
        )}
        {...props}
      />
    );
  },
);

TableHead.displayName = "TableHead";

export type TableCellProps = React.TdHTMLAttributes<HTMLTableCellElement> & {
  numeric?: boolean;
};

const TableCell = React.forwardRef<HTMLTableCellElement, TableCellProps>(
  ({ className, numeric = false, ...props }, ref) => {
    return (
      <td
        ref={ref}
        className={cn(
          "px-4 py-3 align-middle font-mono",
          numeric && "text-right tabular-nums",
          className,
        )}
        {...props}
      />
    );
  },
);

TableCell.displayName = "TableCell";

export type TableCaptionProps = React.HTMLAttributes<HTMLTableCaptionElement>;

const TableCaption = React.forwardRef<HTMLTableCaptionElement, TableCaptionProps>(
  ({ className, ...props }, ref) => {
    return (
      <caption
        ref={ref}
        className={cn("mt-4 font-sans text-sm text-text-muted", className)}
        {...props}
      />
    );
  },
);

TableCaption.displayName = "TableCaption";

export {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableContainer,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
  tableRowVariants,
};
