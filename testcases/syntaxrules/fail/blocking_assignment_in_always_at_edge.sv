module M;
  always @(posedge clk) q = d;
endmodule
////////////////////////////////////////////////////////////////////////////////
module M;
  always @(negedge clk) q = d;
endmodule
////////////////////////////////////////////////////////////////////////////////
module M;
  always @(edge clk) q = d;
endmodule
