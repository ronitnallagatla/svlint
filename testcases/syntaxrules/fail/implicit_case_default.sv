module M;
  always_comb
    case (x)
      1: a = 0; // No implicit or explicit case default
    endcase
endmodule
////////////////////////////////////////////////////////////////////////////////
module M;
  always_comb begin
    y = 0;
    case (x)
      1: y = 1;
      2: begin
        z = 1;
        w = 1;
      end
    endcase
  end
endmodule
////////////////////////////////////////////////////////////////////////////////
module M;
  always_comb begin
    a = 0;
    case (x)
      1: b = 0;
    endcase
  end
endmodule
////////////////////////////////////////////////////////////////////////////////
module M;
  always_comb begin
    q = 0;
    case (x)
      1: p = 1;
      2: q = 1;
      default: q = 1;
    endcase
  end
endmodule
