/**
@port b: this is port b
@param A this is A
*/
module test #( A=1, string B="2", C) (input wire[9:0] a [0:4], b, inout c);

endmodule

/**
 * @brief this is test2
 * @note some notes
 * more notes
 * 
 * and more notes
 * 
 * @author huikan
 * @port a this is port a
 * @port b: this is port b
 * @param C: this is C
 *
 * @example 
 *   assign a = b;
 *   test2 u_test(a,b,c);
 * @wave { signal: [{ name: "Alfa", wave: "01.zx=ud.23.456789" }] }
 */
module test2(a,b,c);
    input a,c;
    wire a;
    input logic[A:0] b [3:0];

    //* this is A
    parameter A[0:5] = 0;
    /** @brief this is B */
    parameter int B;
    parameter C;

    localparam C = 1;
endmodule

module test3();

    task my_task();
    endtask

endmodule