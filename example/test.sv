/**
@port b: this is port b
*/
module test(input wire[9:0] a [0:4], b, inout c);

endmodule

/**
 * @brief this is test2
 * @port a this is port a
 * @port b: this is port b
 * @param C: this is C
 */
module test2(a,b,c);
    input a,c;
    wire a;
    input logic[7:0] b [3:0];

    //* this is A
    parameter A[0:5] = 0;
    /** @brief this is B */
    parameter int B;
    parameter C;

    localparam C = 1;
endmodule