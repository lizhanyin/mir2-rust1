//! 自动生成的调色板数据
//! 由 scripts/convert_palette.py 生成

use super::Color;

/// 传奇2默认调色板
pub const DEFAULT_PALETTE: [Color; 256] = [
    Color { a: 0,   r: 0,   g: 0,   b: 0   }, // index 0:   0x00000000
    Color { a: 255, r: 128, g: 0,   b: 0   }, // index 1:   0x-0800000
    Color { a: 255, r: 0,   g: 128, b: 0   }, // index 2:   0x-0FF8000
    Color { a: 255, r: 128, g: 128, b: 0   }, // index 3:   0x-07F8000
    Color { a: 255, r: 0,   g: 0,   b: 128 }, // index 4:   0x-0FFFF80
    Color { a: 255, r: 128, g: 0,   b: 128 }, // index 5:   0x-07FFF80
    Color { a: 255, r: 0,   g: 128, b: 128 }, // index 6:   0x-0FF7F80
    Color { a: 255, r: 192, g: 192, b: 192 }, // index 7:   0x-03F3F40
    Color { a: 255, r: 85,  g: 128, b: 151 }, // index 8:   0x-0AA7F69
    Color { a: 255, r: 157, g: 185, b: 200 }, // index 9:   0x-0624638
    Color { a: 255, r: 123, g: 115, b: 115 }, // index 10:  0x-0848C8D
    Color { a: 255, r: 45,  g: 41,  b: 41  }, // index 11:  0x-0D2D6D7
    Color { a: 255, r: 90,  g: 82,  b: 82  }, // index 12:  0x-0A5ADAE
    Color { a: 255, r: 99,  g: 90,  b: 90  }, // index 13:  0x-09CA5A6
    Color { a: 255, r: 66,  g: 57,  b: 57  }, // index 14:  0x-0BDC6C7
    Color { a: 255, r: 29,  g: 24,  b: 24  }, // index 15:  0x-0E2E7E8
    Color { a: 255, r: 24,  g: 16,  b: 16  }, // index 16:  0x-0E7EFF0
    Color { a: 255, r: 41,  g: 24,  b: 24  }, // index 17:  0x-0D6E7E8
    Color { a: 255, r: 16,  g: 8,   b: 8   }, // index 18:  0x-0EFF7F8
    Color { a: 255, r: 242, g: 121, b: 113 }, // index 19:  0x-00D868F
    Color { a: 255, r: 225, g: 103, b: 95  }, // index 20:  0x-01E98A1
    Color { a: 255, r: 255, g: 90,  b: 90  }, // index 21:  0x-000A5A6
    Color { a: 255, r: 255, g: 49,  b: 49  }, // index 22:  0x-000CECF
    Color { a: 255, r: 214, g: 90,  b: 82  }, // index 23:  0x-029A5AE
    Color { a: 255, r: 148, g: 16,  b: 0   }, // index 24:  0x-06BF000
    Color { a: 255, r: 148, g: 41,  b: 24  }, // index 25:  0x-06BD6E8
    Color { a: 255, r: 57,  g: 8,   b: 0   }, // index 26:  0x-0C6F800
    Color { a: 255, r: 115, g: 16,  b: 0   }, // index 27:  0x-08CF000
    Color { a: 255, r: 181, g: 24,  b: 0   }, // index 28:  0x-04AE800
    Color { a: 255, r: 189, g: 99,  b: 82  }, // index 29:  0x-0429CAE
    Color { a: 255, r: 66,  g: 24,  b: 16  }, // index 30:  0x-0BDE7F0
    Color { a: 255, r: 255, g: 170, b: 153 }, // index 31:  0x-0005567
    Color { a: 255, r: 90,  g: 16,  b: 0   }, // index 32:  0x-0A5F000
    Color { a: 255, r: 115, g: 57,  b: 41  }, // index 33:  0x-08CC6D7
    Color { a: 255, r: 165, g: 74,  b: 49  }, // index 34:  0x-05AB5CF
    Color { a: 255, r: 148, g: 123, b: 115 }, // index 35:  0x-06B848D
    Color { a: 255, r: 189, g: 82,  b: 49  }, // index 36:  0x-042ADCF
    Color { a: 255, r: 82,  g: 33,  b: 16  }, // index 37:  0x-0ADDEF0
    Color { a: 255, r: 123, g: 49,  b: 24  }, // index 38:  0x-084CEE8
    Color { a: 255, r: 45,  g: 24,  b: 16  }, // index 39:  0x-0D2E7F0
    Color { a: 255, r: 140, g: 74,  b: 49  }, // index 40:  0x-073B5CF
    Color { a: 255, r: 148, g: 41,  b: 0   }, // index 41:  0x-06BD700
    Color { a: 255, r: 189, g: 49,  b: 0   }, // index 42:  0x-042CF00
    Color { a: 255, r: 198, g: 115, b: 82  }, // index 43:  0x-0398CAE
    Color { a: 255, r: 107, g: 49,  b: 24  }, // index 44:  0x-094CEE8
    Color { a: 255, r: 198, g: 107, b: 66  }, // index 45:  0x-03994BE
    Color { a: 255, r: 206, g: 74,  b: 0   }, // index 46:  0x-031B600
    Color { a: 255, r: 165, g: 99,  b: 57  }, // index 47:  0x-05A9CC7
    Color { a: 255, r: 90,  g: 49,  b: 24  }, // index 48:  0x-0A5CEE8
    Color { a: 255, r: 42,  g: 16,  b: 0   }, // index 49:  0x-0D5F000
    Color { a: 255, r: 21,  g: 8,   b: 0   }, // index 50:  0x-0EAF800
    Color { a: 255, r: 58,  g: 24,  b: 0   }, // index 51:  0x-0C5E800
    Color { a: 255, r: 8,   g: 0,   b: 0   }, // index 52:  0x-0F80000
    Color { a: 255, r: 41,  g: 0,   b: 0   }, // index 53:  0x-0D70000
    Color { a: 255, r: 74,  g: 0,   b: 0   }, // index 54:  0x-0B60000
    Color { a: 255, r: 157, g: 0,   b: 0   }, // index 55:  0x-0630000
    Color { a: 255, r: 220, g: 0,   b: 0   }, // index 56:  0x-0240000
    Color { a: 255, r: 222, g: 0,   b: 0   }, // index 57:  0x-0220000
    Color { a: 255, r: 251, g: 0,   b: 0   }, // index 58:  0x-0050000
    Color { a: 255, r: 156, g: 115, b: 82  }, // index 59:  0x-0638CAE
    Color { a: 255, r: 148, g: 107, b: 74  }, // index 60:  0x-06B94B6
    Color { a: 255, r: 115, g: 74,  b: 41  }, // index 61:  0x-08CB5D7
    Color { a: 255, r: 82,  g: 49,  b: 24  }, // index 62:  0x-0ADCEE8
    Color { a: 255, r: 140, g: 74,  b: 24  }, // index 63:  0x-073B5E8
    Color { a: 255, r: 136, g: 68,  b: 17  }, // index 64:  0x-077BBEF
    Color { a: 255, r: 74,  g: 33,  b: 0   }, // index 65:  0x-0B5DF00
    Color { a: 255, r: 33,  g: 24,  b: 16  }, // index 66:  0x-0DEE7F0
    Color { a: 255, r: 214, g: 148, b: 90  }, // index 67:  0x-0296BA6
    Color { a: 255, r: 198, g: 107, b: 33  }, // index 68:  0x-03994DF
    Color { a: 255, r: 239, g: 107, b: 0   }, // index 69:  0x-0109500
    Color { a: 255, r: 255, g: 119, b: 0   }, // index 70:  0x-0008900
    Color { a: 255, r: 165, g: 148, b: 132 }, // index 71:  0x-05A6B7C
    Color { a: 255, r: 66,  g: 49,  b: 33  }, // index 72:  0x-0BDCEDF
    Color { a: 255, r: 24,  g: 16,  b: 8   }, // index 73:  0x-0E7EFF8
    Color { a: 255, r: 41,  g: 24,  b: 8   }, // index 74:  0x-0D6E7F8
    Color { a: 255, r: 33,  g: 16,  b: 0   }, // index 75:  0x-0DEF000
    Color { a: 255, r: 57,  g: 41,  b: 24  }, // index 76:  0x-0C6D6E8
    Color { a: 255, r: 140, g: 99,  b: 57  }, // index 77:  0x-0739CC7
    Color { a: 255, r: 66,  g: 41,  b: 16  }, // index 78:  0x-0BDD6F0
    Color { a: 255, r: 107, g: 66,  b: 24  }, // index 79:  0x-094BDE8
    Color { a: 255, r: 123, g: 74,  b: 24  }, // index 80:  0x-084B5E8
    Color { a: 255, r: 148, g: 74,  b: 0   }, // index 81:  0x-06BB600
    Color { a: 255, r: 140, g: 132, b: 123 }, // index 82:  0x-0737B85
    Color { a: 255, r: 107, g: 99,  b: 90  }, // index 83:  0x-0949CA6
    Color { a: 255, r: 74,  g: 66,  b: 57  }, // index 84:  0x-0B5BDC7
    Color { a: 255, r: 41,  g: 33,  b: 24  }, // index 85:  0x-0D6DEE8
    Color { a: 255, r: 70,  g: 57,  b: 41  }, // index 86:  0x-0B9C6D7
    Color { a: 255, r: 181, g: 165, b: 148 }, // index 87:  0x-04A5A6C
    Color { a: 255, r: 123, g: 107, b: 90  }, // index 88:  0x-08494A6
    Color { a: 255, r: 206, g: 177, b: 148 }, // index 89:  0x-0314E6C
    Color { a: 255, r: 165, g: 140, b: 115 }, // index 90:  0x-05A738D
    Color { a: 255, r: 140, g: 115, b: 90  }, // index 91:  0x-0738CA6
    Color { a: 255, r: 181, g: 148, b: 115 }, // index 92:  0x-04A6B8D
    Color { a: 255, r: 214, g: 165, b: 115 }, // index 93:  0x-0295A8D
    Color { a: 255, r: 239, g: 165, b: 74  }, // index 94:  0x-0105AB6
    Color { a: 255, r: 239, g: 198, b: 140 }, // index 95:  0x-0103974
    Color { a: 255, r: 123, g: 99,  b: 66  }, // index 96:  0x-0849CBE
    Color { a: 255, r: 107, g: 86,  b: 57  }, // index 97:  0x-094A9C7
    Color { a: 255, r: 189, g: 148, b: 90  }, // index 98:  0x-0426BA6
    Color { a: 255, r: 99,  g: 57,  b: 0   }, // index 99:  0x-09CC700
    Color { a: 255, r: 214, g: 198, b: 173 }, // index 100: 0x-0293953
    Color { a: 255, r: 82,  g: 66,  b: 41  }, // index 101: 0x-0ADBDD7
    Color { a: 255, r: 148, g: 99,  b: 24  }, // index 102: 0x-06B9CE8
    Color { a: 255, r: 239, g: 214, b: 173 }, // index 103: 0x-0102953
    Color { a: 255, r: 165, g: 140, b: 99  }, // index 104: 0x-05A739D
    Color { a: 255, r: 99,  g: 90,  b: 74  }, // index 105: 0x-09CA5B6
    Color { a: 255, r: 189, g: 165, b: 123 }, // index 106: 0x-0425A85
    Color { a: 255, r: 90,  g: 66,  b: 24  }, // index 107: 0x-0A5BDE8
    Color { a: 255, r: 189, g: 140, b: 49  }, // index 108: 0x-04273CF
    Color { a: 255, r: 53,  g: 49,  b: 41  }, // index 109: 0x-0CACED7
    Color { a: 255, r: 148, g: 132, b: 99  }, // index 110: 0x-06B7B9D
    Color { a: 255, r: 123, g: 107, b: 74  }, // index 111: 0x-08494B6
    Color { a: 255, r: 165, g: 140, b: 90  }, // index 112: 0x-05A73A6
    Color { a: 255, r: 90,  g: 74,  b: 41  }, // index 113: 0x-0A5B5D7
    Color { a: 255, r: 156, g: 123, b: 57  }, // index 114: 0x-06384C7
    Color { a: 255, r: 66,  g: 49,  b: 16  }, // index 115: 0x-0BDCEF0
    Color { a: 255, r: 239, g: 173, b: 33  }, // index 116: 0x-01052DF
    Color { a: 255, r: 24,  g: 16,  b: 0   }, // index 117: 0x-0E7F000
    Color { a: 255, r: 41,  g: 33,  b: 0   }, // index 118: 0x-0D6DF00
    Color { a: 255, r: 156, g: 107, b: 0   }, // index 119: 0x-0639500
    Color { a: 255, r: 148, g: 132, b: 90  }, // index 120: 0x-06B7BA6
    Color { a: 255, r: 82,  g: 66,  b: 24  }, // index 121: 0x-0ADBDE8
    Color { a: 255, r: 107, g: 90,  b: 41  }, // index 122: 0x-094A5D7
    Color { a: 255, r: 123, g: 99,  b: 33  }, // index 123: 0x-0849CDF
    Color { a: 255, r: 156, g: 123, b: 33  }, // index 124: 0x-06384DF
    Color { a: 255, r: 222, g: 165, b: 0   }, // index 125: 0x-0215B00
    Color { a: 255, r: 90,  g: 82,  b: 57  }, // index 126: 0x-0A5ADC7
    Color { a: 255, r: 49,  g: 41,  b: 16  }, // index 127: 0x-0CED6F0
    Color { a: 255, r: 206, g: 189, b: 123 }, // index 128: 0x-0314285
    Color { a: 255, r: 99,  g: 90,  b: 57  }, // index 129: 0x-09CA5C7
    Color { a: 255, r: 148, g: 132, b: 74  }, // index 130: 0x-06B7BB6
    Color { a: 255, r: 198, g: 165, b: 41  }, // index 131: 0x-0395AD7
    Color { a: 255, r: 16,  g: 156, b: 24  }, // index 132: 0x-0EF63E8
    Color { a: 255, r: 66,  g: 140, b: 74  }, // index 133: 0x-0BD73B6
    Color { a: 255, r: 49,  g: 140, b: 66  }, // index 134: 0x-0CE73BE
    Color { a: 255, r: 16,  g: 148, b: 41  }, // index 135: 0x-0EF6BD7
    Color { a: 255, r: 8,   g: 24,  b: 16  }, // index 136: 0x-0F7E7F0
    Color { a: 255, r: 8,   g: 24,  b: 24  }, // index 137: 0x-0F7E7E8
    Color { a: 255, r: 8,   g: 41,  b: 16  }, // index 138: 0x-0F7D6F0
    Color { a: 255, r: 24,  g: 66,  b: 41  }, // index 139: 0x-0E7BDD7
    Color { a: 255, r: 165, g: 181, b: 173 }, // index 140: 0x-05A4A53
    Color { a: 255, r: 107, g: 115, b: 115 }, // index 141: 0x-0948C8D
    Color { a: 255, r: 24,  g: 41,  b: 41  }, // index 142: 0x-0E7D6D7
    Color { a: 255, r: 24,  g: 66,  b: 74  }, // index 143: 0x-0E7BDB6
    Color { a: 255, r: 49,  g: 66,  b: 74  }, // index 144: 0x-0CEBDB6
    Color { a: 255, r: 99,  g: 198, b: 222 }, // index 145: 0x-09C3922
    Color { a: 255, r: 68,  g: 221, b: 255 }, // index 146: 0x-0BB2201
    Color { a: 255, r: 140, g: 214, b: 239 }, // index 147: 0x-0732911
    Color { a: 255, r: 115, g: 107, b: 57  }, // index 148: 0x-08C94C7
    Color { a: 255, r: 247, g: 222, b: 57  }, // index 149: 0x-00821C7
    Color { a: 255, r: 247, g: 239, b: 140 }, // index 150: 0x-0081074
    Color { a: 255, r: 247, g: 231, b: 0   }, // index 151: 0x-0081900
    Color { a: 255, r: 107, g: 107, b: 90  }, // index 152: 0x-09494A6
    Color { a: 255, r: 90,  g: 140, b: 165 }, // index 153: 0x-0A5735B
    Color { a: 255, r: 57,  g: 181, b: 239 }, // index 154: 0x-0C64A11
    Color { a: 255, r: 74,  g: 156, b: 206 }, // index 155: 0x-0B56332
    Color { a: 255, r: 49,  g: 132, b: 181 }, // index 156: 0x-0CE7B4B
    Color { a: 255, r: 49,  g: 82,  b: 107 }, // index 157: 0x-0CEAD95
    Color { a: 255, r: 222, g: 222, b: 214 }, // index 158: 0x-021212A
    Color { a: 255, r: 189, g: 189, b: 181 }, // index 159: 0x-042424B
    Color { a: 255, r: 140, g: 140, b: 132 }, // index 160: 0x-073737C
    Color { a: 255, r: 247, g: 247, b: 222 }, // index 161: 0x-0080822
    Color { a: 255, r: 0,   g: 8,   b: 24  }, // index 162: 0x-0FFF7E8
    Color { a: 255, r: 8,   g: 24,  b: 57  }, // index 163: 0x-0F7E7C7
    Color { a: 255, r: 8,   g: 16,  b: 41  }, // index 164: 0x-0F7EFD7
    Color { a: 255, r: 8,   g: 24,  b: 0   }, // index 165: 0x-0F7E800
    Color { a: 255, r: 8,   g: 41,  b: 0   }, // index 166: 0x-0F7D700
    Color { a: 255, r: 0,   g: 82,  b: 165 }, // index 167: 0x-0FFAD5B
    Color { a: 255, r: 0,   g: 123, b: 222 }, // index 168: 0x-0FF8422
    Color { a: 255, r: 16,  g: 41,  b: 74  }, // index 169: 0x-0EFD6B6
    Color { a: 255, r: 16,  g: 57,  b: 107 }, // index 170: 0x-0EFC695
    Color { a: 255, r: 16,  g: 82,  b: 140 }, // index 171: 0x-0EFAD74
    Color { a: 255, r: 33,  g: 90,  b: 165 }, // index 172: 0x-0DEA55B
    Color { a: 255, r: 16,  g: 49,  b: 90  }, // index 173: 0x-0EFCEA6
    Color { a: 255, r: 16,  g: 66,  b: 132 }, // index 174: 0x-0EFBD7C
    Color { a: 255, r: 49,  g: 82,  b: 132 }, // index 175: 0x-0CEAD7C
    Color { a: 255, r: 24,  g: 33,  b: 49  }, // index 176: 0x-0E7DECF
    Color { a: 255, r: 74,  g: 90,  b: 123 }, // index 177: 0x-0B5A585
    Color { a: 255, r: 82,  g: 107, b: 165 }, // index 178: 0x-0AD945B
    Color { a: 255, r: 41,  g: 57,  b: 99  }, // index 179: 0x-0D6C69D
    Color { a: 255, r: 16,  g: 74,  b: 222 }, // index 180: 0x-0EFB522
    Color { a: 255, r: 41,  g: 41,  b: 33  }, // index 181: 0x-0D6D6DF
    Color { a: 255, r: 74,  g: 74,  b: 57  }, // index 182: 0x-0B5B5C7
    Color { a: 255, r: 41,  g: 41,  b: 24  }, // index 183: 0x-0D6D6E8
    Color { a: 255, r: 74,  g: 74,  b: 41  }, // index 184: 0x-0B5B5D7
    Color { a: 255, r: 123, g: 123, b: 66  }, // index 185: 0x-08484BE
    Color { a: 255, r: 156, g: 156, b: 74  }, // index 186: 0x-06363B6
    Color { a: 255, r: 90,  g: 90,  b: 41  }, // index 187: 0x-0A5A5D7
    Color { a: 255, r: 66,  g: 66,  b: 20  }, // index 188: 0x-0BDBDEC
    Color { a: 255, r: 57,  g: 57,  b: 0   }, // index 189: 0x-0C6C700
    Color { a: 255, r: 89,  g: 89,  b: 0   }, // index 190: 0x-0A6A700
    Color { a: 255, r: 202, g: 53,  b: 44  }, // index 191: 0x-035CAD4
    Color { a: 255, r: 107, g: 115, b: 33  }, // index 192: 0x-0948CDF
    Color { a: 255, r: 41,  g: 49,  b: 0   }, // index 193: 0x-0D6CF00
    Color { a: 255, r: 49,  g: 57,  b: 16  }, // index 194: 0x-0CEC6F0
    Color { a: 255, r: 49,  g: 57,  b: 24  }, // index 195: 0x-0CEC6E8
    Color { a: 255, r: 66,  g: 74,  b: 0   }, // index 196: 0x-0BDB600
    Color { a: 255, r: 82,  g: 99,  b: 24  }, // index 197: 0x-0AD9CE8
    Color { a: 255, r: 90,  g: 115, b: 41  }, // index 198: 0x-0A58CD7
    Color { a: 255, r: 49,  g: 74,  b: 24  }, // index 199: 0x-0CEB5E8
    Color { a: 255, r: 24,  g: 33,  b: 0   }, // index 200: 0x-0E7DF00
    Color { a: 255, r: 24,  g: 49,  b: 0   }, // index 201: 0x-0E7CF00
    Color { a: 255, r: 24,  g: 57,  b: 16  }, // index 202: 0x-0E7C6F0
    Color { a: 255, r: 99,  g: 132, b: 74  }, // index 203: 0x-09C7BB6
    Color { a: 255, r: 107, g: 189, b: 74  }, // index 204: 0x-09442B6
    Color { a: 255, r: 99,  g: 181, b: 74  }, // index 205: 0x-09C4AB6
    Color { a: 255, r: 99,  g: 189, b: 74  }, // index 206: 0x-09C42B6
    Color { a: 255, r: 90,  g: 156, b: 74  }, // index 207: 0x-0A563B6
    Color { a: 255, r: 74,  g: 140, b: 57  }, // index 208: 0x-0B573C7
    Color { a: 255, r: 99,  g: 198, b: 74  }, // index 209: 0x-09C39B6
    Color { a: 255, r: 99,  g: 214, b: 74  }, // index 210: 0x-09C29B6
    Color { a: 255, r: 82,  g: 132, b: 74  }, // index 211: 0x-0AD7BB6
    Color { a: 255, r: 49,  g: 115, b: 41  }, // index 212: 0x-0CE8CD7
    Color { a: 255, r: 99,  g: 198, b: 90  }, // index 213: 0x-09C39A6
    Color { a: 255, r: 82,  g: 189, b: 74  }, // index 214: 0x-0AD42B6
    Color { a: 255, r: 16,  g: 255, b: 0   }, // index 215: 0x-0EF0100
    Color { a: 255, r: 24,  g: 41,  b: 24  }, // index 216: 0x-0E7D6E8
    Color { a: 255, r: 74,  g: 136, b: 74  }, // index 217: 0x-0B577B6
    Color { a: 255, r: 74,  g: 231, b: 74  }, // index 218: 0x-0B518B6
    Color { a: 255, r: 0,   g: 90,  b: 0   }, // index 219: 0x-0FFA600
    Color { a: 255, r: 0,   g: 136, b: 0   }, // index 220: 0x-0FF7800
    Color { a: 255, r: 0,   g: 148, b: 0   }, // index 221: 0x-0FF6C00
    Color { a: 255, r: 0,   g: 222, b: 0   }, // index 222: 0x-0FF2200
    Color { a: 255, r: 0,   g: 238, b: 0   }, // index 223: 0x-0FF1200
    Color { a: 255, r: 0,   g: 251, b: 0   }, // index 224: 0x-0FF0500
    Color { a: 255, r: 74,  g: 90,  b: 148 }, // index 225: 0x-0B5A56C
    Color { a: 255, r: 99,  g: 115, b: 181 }, // index 226: 0x-09C8C4B
    Color { a: 255, r: 123, g: 140, b: 214 }, // index 227: 0x-084732A
    Color { a: 255, r: 107, g: 123, b: 214 }, // index 228: 0x-094842A
    Color { a: 255, r: 119, g: 136, b: 255 }, // index 229: 0x-0887701
    Color { a: 255, r: 198, g: 198, b: 206 }, // index 230: 0x-0393932
    Color { a: 255, r: 148, g: 148, b: 156 }, // index 231: 0x-06B6B64
    Color { a: 255, r: 156, g: 148, b: 198 }, // index 232: 0x-0636B3A
    Color { a: 255, r: 49,  g: 49,  b: 57  }, // index 233: 0x-0CECEC7
    Color { a: 255, r: 41,  g: 24,  b: 132 }, // index 234: 0x-0D6E77C
    Color { a: 255, r: 24,  g: 0,   b: 132 }, // index 235: 0x-0E7FF7C
    Color { a: 255, r: 74,  g: 66,  b: 82  }, // index 236: 0x-0B5BDAE
    Color { a: 255, r: 82,  g: 66,  b: 123 }, // index 237: 0x-0ADBD85
    Color { a: 255, r: 99,  g: 90,  b: 115 }, // index 238: 0x-09CA58D
    Color { a: 255, r: 206, g: 181, b: 247 }, // index 239: 0x-0314A09
    Color { a: 255, r: 140, g: 123, b: 156 }, // index 240: 0x-0738464
    Color { a: 255, r: 119, g: 34,  b: 204 }, // index 241: 0x-088DD34
    Color { a: 255, r: 221, g: 170, b: 255 }, // index 242: 0x-0225501
    Color { a: 255, r: 240, g: 180, b: 42  }, // index 243: 0x-00F4BD6
    Color { a: 255, r: 223, g: 0,   b: 159 }, // index 244: 0x-020FF61
    Color { a: 255, r: 227, g: 23,  b: 179 }, // index 245: 0x-01CE84D
    Color { a: 255, r: 255, g: 251, b: 240 }, // index 246: 0x-0000410
    Color { a: 255, r: 160, g: 160, b: 164 }, // index 247: 0x-05F5F5C
    Color { a: 255, r: 128, g: 128, b: 128 }, // index 248: 0x-07F7F80
    Color { a: 255, r: 255, g: 0,   b: 0   }, // index 249: 0x-0010000
    Color { a: 255, r: 0,   g: 255, b: 0   }, // index 250: 0x-0FF0100
    Color { a: 255, r: 255, g: 255, b: 0   }, // index 251: 0x-0000100
    Color { a: 255, r: 0,   g: 0,   b: 255 }, // index 252: 0x-0FFFF01
    Color { a: 255, r: 255, g: 0,   b: 255 }, // index 253: 0x-000FF01
    Color { a: 255, r: 0,   g: 255, b: 255 }, // index 254: 0x-0FF0001
    Color { a: 255, r: 255, g: 255, b: 255 }, // index 255: 0x-0000001
];

/// 快速查找表 (u32格式的颜色值)
pub const PALETTE_U32: [u32; 256] = [
    0x00000000, 0xFF000080, 0xFF008000, 0xFF008080, 0xFF800000, 0xFF800080, 0xFF808000, 0xFFC0C0C0, 0xFF978055, 0xFFC8B99D, 0xFF73737B, 0xFF29292D, 0xFF52525A, 0xFF5A5A63, 0xFF393942, 0xFF18181D,
    0xFF101018, 0xFF181829, 0xFF080810, 0xFF7179F2, 0xFF5F67E1, 0xFF5A5AFF, 0xFF3131FF, 0xFF525AD6, 0xFF001094, 0xFF182994, 0xFF000839, 0xFF001073, 0xFF0018B5, 0xFF5263BD, 0xFF101842, 0xFF99AAFF,
    0xFF00105A, 0xFF293973, 0xFF314AA5, 0xFF737B94, 0xFF3152BD, 0xFF102152, 0xFF18317B, 0xFF10182D, 0xFF314A8C, 0xFF002994, 0xFF0031BD, 0xFF5273C6, 0xFF18316B, 0xFF426BC6, 0xFF004ACE, 0xFF3963A5,
    0xFF18315A, 0xFF00102A, 0xFF000815, 0xFF00183A, 0xFF000008, 0xFF000029, 0xFF00004A, 0xFF00009D, 0xFF0000DC, 0xFF0000DE, 0xFF0000FB, 0xFF52739C, 0xFF4A6B94, 0xFF294A73, 0xFF183152, 0xFF184A8C,
    0xFF114488, 0xFF00214A, 0xFF101821, 0xFF5A94D6, 0xFF216BC6, 0xFF006BEF, 0xFF0077FF, 0xFF8494A5, 0xFF213142, 0xFF081018, 0xFF081829, 0xFF001021, 0xFF182939, 0xFF39638C, 0xFF102942, 0xFF18426B,
    0xFF184A7B, 0xFF004A94, 0xFF7B848C, 0xFF5A636B, 0xFF39424A, 0xFF182129, 0xFF293946, 0xFF94A5B5, 0xFF5A6B7B, 0xFF94B1CE, 0xFF738CA5, 0xFF5A738C, 0xFF7394B5, 0xFF73A5D6, 0xFF4AA5EF, 0xFF8CC6EF,
    0xFF42637B, 0xFF39566B, 0xFF5A94BD, 0xFF003963, 0xFFADC6D6, 0xFF294252, 0xFF186394, 0xFFADD6EF, 0xFF638CA5, 0xFF4A5A63, 0xFF7BA5BD, 0xFF18425A, 0xFF318CBD, 0xFF293135, 0xFF638494, 0xFF4A6B7B,
    0xFF5A8CA5, 0xFF294A5A, 0xFF397B9C, 0xFF103142, 0xFF21ADEF, 0xFF001018, 0xFF002129, 0xFF006B9C, 0xFF5A8494, 0xFF184252, 0xFF295A6B, 0xFF21637B, 0xFF217B9C, 0xFF00A5DE, 0xFF39525A, 0xFF102931,
    0xFF7BBDCE, 0xFF395A63, 0xFF4A8494, 0xFF29A5C6, 0xFF189C10, 0xFF4A8C42, 0xFF428C31, 0xFF299410, 0xFF101808, 0xFF181808, 0xFF102908, 0xFF294218, 0xFFADB5A5, 0xFF73736B, 0xFF292918, 0xFF4A4218,
    0xFF4A4231, 0xFFDEC663, 0xFFFFDD44, 0xFFEFD68C, 0xFF396B73, 0xFF39DEF7, 0xFF8CEFF7, 0xFF00E7F7, 0xFF5A6B6B, 0xFFA58C5A, 0xFFEFB539, 0xFFCE9C4A, 0xFFB58431, 0xFF6B5231, 0xFFD6DEDE, 0xFFB5BDBD,
    0xFF848C8C, 0xFFDEF7F7, 0xFF180800, 0xFF391808, 0xFF291008, 0xFF001808, 0xFF002908, 0xFFA55200, 0xFFDE7B00, 0xFF4A2910, 0xFF6B3910, 0xFF8C5210, 0xFFA55A21, 0xFF5A3110, 0xFF844210, 0xFF845231,
    0xFF312118, 0xFF7B5A4A, 0xFFA56B52, 0xFF633929, 0xFFDE4A10, 0xFF212929, 0xFF394A4A, 0xFF182929, 0xFF294A4A, 0xFF427B7B, 0xFF4A9C9C, 0xFF295A5A, 0xFF144242, 0xFF003939, 0xFF005959, 0xFF2C35CA,
    0xFF21736B, 0xFF003129, 0xFF103931, 0xFF183931, 0xFF004A42, 0xFF186352, 0xFF29735A, 0xFF184A31, 0xFF002118, 0xFF003118, 0xFF103918, 0xFF4A8463, 0xFF4ABD6B, 0xFF4AB563, 0xFF4ABD63, 0xFF4A9C5A,
    0xFF398C4A, 0xFF4AC663, 0xFF4AD663, 0xFF4A8452, 0xFF297331, 0xFF5AC663, 0xFF4ABD52, 0xFF00FF10, 0xFF182918, 0xFF4A884A, 0xFF4AE74A, 0xFF005A00, 0xFF008800, 0xFF009400, 0xFF00DE00, 0xFF00EE00,
    0xFF00FB00, 0xFF945A4A, 0xFFB57363, 0xFFD68C7B, 0xFFD67B6B, 0xFFFF8877, 0xFFCEC6C6, 0xFF9C9494, 0xFFC6949C, 0xFF393131, 0xFF841829, 0xFF840018, 0xFF52424A, 0xFF7B4252, 0xFF735A63, 0xFFF7B5CE,
    0xFF9C7B8C, 0xFFCC2277, 0xFFFFAADD, 0xFF2AB4F0, 0xFF9F00DF, 0xFFB317E3, 0xFFF0FBFF, 0xFFA4A0A0, 0xFF808080, 0xFF0000FF, 0xFF00FF00, 0xFF00FFFF, 0xFFFF0000, 0xFFFF00FF, 0xFFFFFF00, 0xFFFFFFFF,
];
