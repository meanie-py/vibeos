/*
 * Intel ACPI Component Architecture
 * AML/ASL+ Disassembler version 20230628 (64-bit version)
 * Copyright (c) 2000 - 2023 Intel Corporation
 *
 * Disassembling to symbolic ASL+ operators
 *
 * Disassembly of dsdt.aml, Tue Oct  6 22:31:07 2026
 *
 * Original Table Header:
 *     Signature        "DSDT"
 *     Length           0x000014DD (5341)
 *     Revision         0x02
 *     Checksum         0x4A
 *     OEM ID           "RPIFDN"
 *     OEM Table ID     "RPI5    "
 *     OEM Revision     0x00000002 (2)
 *     Compiler ID      "INTL"
 *     Compiler Version 0x20200925 (538970405)
 */
DefinitionBlock ("", "DSDT", 2, "RPIFDN", "RPI5    ", 0x00000002)
{
    Scope (_SB)
    {
        Device (CPU0)
        {
            Name (_HID, "ACPI0007" /* Processor Device */)  // _HID: Hardware ID
            Name (_UID, Zero)  // _UID: Unique ID
            Name (_STA, 0x0F)  // _STA: Status
        }

        Device (CPU1)
        {
            Name (_HID, "ACPI0007" /* Processor Device */)  // _HID: Hardware ID
            Name (_UID, One)  // _UID: Unique ID
            Name (_STA, 0x0F)  // _STA: Status
        }

        Device (CPU2)
        {
            Name (_HID, "ACPI0007" /* Processor Device */)  // _HID: Hardware ID
            Name (_UID, 0x02)  // _UID: Unique ID
            Name (_STA, 0x0F)  // _STA: Status
        }

        Device (CPU3)
        {
            Name (_HID, "ACPI0007" /* Processor Device */)  // _HID: Hardware ID
            Name (_UID, 0x03)  // _UID: Unique ID
            Name (_STA, 0x0F)  // _STA: Status
        }

        Device (SOCB)
        {
            Name (_HID, "ACPI0004" /* Module Device */)  // _HID: Hardware ID
            Name (_UID, Zero)  // _UID: Unique ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
            {
                Name (RBUF, ResourceTemplate ()
                {
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y00, AddressRangeMemory, TypeStatic)
                })
                CreateQWordField (RBUF, \_SB.SOCB._CRS._Y00._MIN, MI00)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.SOCB._CRS._Y00._MAX, MA00)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.SOCB._CRS._Y00._TRA, TR00)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.SOCB._CRS._Y00._LEN, LE00)  // _LEN: Length
                LE00 = 0x04000000
                MI00 = 0x000000107C000000
                TR00 = Zero
                MA00 = ((MI00 + LE00) - One)
                Return (RBUF) /* \_SB_.SOCB._CRS.RBUF */
            }

            Name (_DMA, ResourceTemplate ()  // _DMA: Direct Memory Access
            {
                QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                    0x0000000000000000, // Granularity
                    0x00000000C0000000, // Range Minimum
                    0x00000000FFFFFFFF, // Range Maximum
                    0xFFFFFFFF40000000, // Translation Offset
                    0x0000000040000000, // Length
                    ,, , AddressRangeMemory, TypeStatic)
            })
            Device (URT0)
            {
                Name (_HID, "ARMH0011")  // _HID: Hardware ID
                Name (_UID, Zero)  // _UID: Unique ID
                Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
                Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
                {
                    Name (RBUF, ResourceTemplate ()
                    {
                        QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                            0x0000000000000000, // Granularity
                            0x0000000000000000, // Range Minimum
                            0x0000000000000000, // Range Maximum
                            0x0000000000000000, // Translation Offset
                            0x0000000000000001, // Length
                            ,, _Y01, AddressRangeMemory, TypeStatic)
                        Interrupt (ResourceConsumer, Level, ActiveHigh, Exclusive, ,, )
                        {
                            0x00000099,
                        }
                    })
                    CreateQWordField (RBUF, \_SB.SOCB.URT0._CRS._Y01._MIN, MI00)  // _MIN: Minimum Base Address
                    CreateQWordField (RBUF, \_SB.SOCB.URT0._CRS._Y01._MAX, MA00)  // _MAX: Maximum Base Address
                    CreateQWordField (RBUF, \_SB.SOCB.URT0._CRS._Y01._TRA, TR00)  // _TRA: Translation
                    CreateQWordField (RBUF, \_SB.SOCB.URT0._CRS._Y01._LEN, LE00)  // _LEN: Length
                    LE00 = 0x0200
                    MI00 = 0x000000107D001000
                    TR00 = Zero
                    MA00 = ((MI00 + LE00) - One)
                    Return (RBUF) /* \_SB_.SOCB.URT0._CRS.RBUF */
                }

                Name (_DSD, Package (0x02)  // _DSD: Device-Specific Data
                {
                    ToUUID ("daffd814-6eba-4d8c-8a91-bc9bbf4aa301") /* Device Properties for _DSD */,
                    Package (0x01)
                    {
                        Package (0x02)
                        {
                            "clock-frequency",
                            0x029F6300
                        }
                    }
                })
            }
        }

        Name (PBMA, 0xFF)
        Name (BB32, 0x00000000C0000000)
        Name (MS32, 0x0000000000500000)
        Device (PCI0)
        {
            Name (_SEG, Zero)  // _SEG: PCI Segment
            Name (_STA, 0x00)  // _STA: Status
            Name (CFGB, 0x0000001000100000)
            Name (CFGS, 0x9310)
            Name (MB32, 0x0000001700000000)
            Name (MB64, 0x0000001400000000)
            Name (MS64, 0x0000000300000000)
            Name (_PRT, Package (0x04)  // _PRT: PCI Routing Table
            {
                Package (0x04)
                {
                    0xFFFF,
                    Zero,
                    Zero,
                    0xF1
                },

                Package (0x04)
                {
                    0xFFFF,
                    One,
                    Zero,
                    0xF2
                },

                Package (0x04)
                {
                    0xFFFF,
                    0x02,
                    Zero,
                    0xF3
                },

                Package (0x04)
                {
                    0xFFFF,
                    0x03,
                    Zero,
                    0xF4
                }
            })
            Name (_HID, "PNP0A08" /* PCI Express Bus */)  // _HID: Hardware ID
            Name (_CID, "PNP0A03" /* PCI Bus */)  // _CID: Compatible ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Name (_BBN, Zero)  // _BBN: BIOS Bus Number
            Method (_UID, 0, NotSerialized)  // _UID: Unique ID
            {
                Return (_SEG) /* \_SB_.PCI0._SEG */
            }

            Method (_INI, 0, Serialized)  // _INI: Initialize
            {
                OperationRegion (OCFG, SystemMemory, (CFGB + 0x9000), 0x04)
                Field (OCFG, DWordAcc, NoLock, Preserve)
                {
                    CFGI,   32
                }

                CFGI = 0x00100000
            }

            Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
            {
                Name (RBUF, ResourceTemplate ()
                {
                    WordBusNumber (ResourceProducer, MinFixed, MaxFixed, PosDecode,
                        0x0000,             // Granularity
                        0x0000,             // Range Minimum
                        0x0000,             // Range Maximum
                        0x0000,             // Translation Offset
                        0x0001,             // Length
                        ,, _Y02)
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y03, AddressRangeMemory, TypeStatic)
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y04, AddressRangeMemory, TypeStatic)
                })
                CreateWordField (RBUF, \_SB.PCI0._CRS._Y02._MIN, MI00)  // _MIN: Minimum Base Address
                CreateWordField (RBUF, \_SB.PCI0._CRS._Y02._MAX, MA00)  // _MAX: Maximum Base Address
                CreateWordField (RBUF, \_SB.PCI0._CRS._Y02._TRA, TR00)  // _TRA: Translation
                CreateWordField (RBUF, \_SB.PCI0._CRS._Y02._LEN, LE00)  // _LEN: Length
                LE00 = ((PBMA - _BBN) + One)
                MI00 = _BBN /* \_SB_.PCI0._BBN */
                TR00 = Zero
                MA00 = ((MI00 + LE00) - One)
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y03._MIN, MI01)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y03._MAX, MA01)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y03._TRA, TR01)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y03._LEN, LE01)  // _LEN: Length
                LE01 = MS32 /* \_SB_.MS32 */
                MI01 = BB32 /* \_SB_.BB32 */
                TR01 = (MB32 - BB32) /* \_SB_.BB32 */
                MA01 = ((MI01 + LE01) - One)
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y04._MIN, MI02)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y04._MAX, MA02)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y04._TRA, TR02)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.PCI0._CRS._Y04._LEN, LE02)  // _LEN: Length
                LE02 = MS64 /* \_SB_.PCI0.MS64 */
                MI02 = MB64 /* \_SB_.PCI0.MB64 */
                TR02 = Zero
                MA02 = ((MI02 + LE02) - One)
                Return (RBUF) /* \_SB_.PCI0._CRS.RBUF */
            }

            Device (RES0)
            {
                Name (_HID, "AMZN0001")  // _HID: Hardware ID
                Name (_CID, "PNP0C02" /* PNP Motherboard Resources */)  // _CID: Compatible ID
                Method (_UID, 0, NotSerialized)  // _UID: Unique ID
                {
                    Return (_SEG) /* \_SB_.PCI0._SEG */
                }

                Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
                {
                    Name (RBUF, ResourceTemplate ()
                    {
                        QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                            0x0000000000000000, // Granularity
                            0x0000000000000000, // Range Minimum
                            0x0000000000000000, // Range Maximum
                            0x0000000000000000, // Translation Offset
                            0x0000000000000001, // Length
                            ,, _Y05, AddressRangeMemory, TypeStatic)
                    })
                    CreateQWordField (RBUF, \_SB.PCI0.RES0._CRS._Y05._MIN, MI00)  // _MIN: Minimum Base Address
                    CreateQWordField (RBUF, \_SB.PCI0.RES0._CRS._Y05._MAX, MA00)  // _MAX: Maximum Base Address
                    CreateQWordField (RBUF, \_SB.PCI0.RES0._CRS._Y05._TRA, TR00)  // _TRA: Translation
                    CreateQWordField (RBUF, \_SB.PCI0.RES0._CRS._Y05._LEN, LE00)  // _LEN: Length
                    LE00 = 0x1000
                    MI00 = (CFGB + 0x8000)
                    TR00 = Zero
                    MA00 = ((MI00 + LE00) - One)
                    Return (RBUF) /* \_SB_.PCI0.RES0._CRS.RBUF */
                }
            }

            Name (SUPP, Zero)
            Name (CTRL, Zero)
            Method (_OSC, 4, NotSerialized)  // _OSC: Operating System Capabilities
            {
                If ((Arg0 == ToUUID ("33db4d5b-1ff7-401c-9657-7441c03dd766") /* PCI Host Bridge Device */))
                {
                    CreateDWordField (Arg3, Zero, CDW1)
                    CreateDWordField (Arg3, 0x04, CDW2)
                    CreateDWordField (Arg3, 0x08, CDW3)
                    SUPP = CDW2 /* \_SB_.PCI0._OSC.CDW2 */
                    CTRL = CDW3 /* \_SB_.PCI0._OSC.CDW3 */
                    CTRL &= 0x1E
                    CTRL &= 0x1D
                    If ((Arg1 != One))
                    {
                        CDW1 |= 0x08
                    }

                    If ((CDW3 != CTRL))
                    {
                        CDW1 |= 0x10
                    }

                    CDW3 = CTRL /* \_SB_.PCI0.CTRL */
                    Return (Arg3)
                }
                Else
                {
                    CDW1 |= 0x04
                    Return (Arg3)
                }
            }
        }

        Device (PCI1)
        {
            Name (_SEG, One)  // _SEG: PCI Segment
            Name (_STA, 0x0F)  // _STA: Status
            Name (CFGB, 0x0000001000110000)
            Name (CFGS, 0x9310)
            Name (MB32, 0x0000001B00000000)
            Name (MB64, 0x0000001800000000)
            Name (MS64, 0x0000000300000000)
            Name (_PRT, Package (0x04)  // _PRT: PCI Routing Table
            {
                Package (0x04)
                {
                    0xFFFF,
                    Zero,
                    Zero,
                    0xFB
                },

                Package (0x04)
                {
                    0xFFFF,
                    One,
                    Zero,
                    0xFC
                },

                Package (0x04)
                {
                    0xFFFF,
                    0x02,
                    Zero,
                    0xFD
                },

                Package (0x04)
                {
                    0xFFFF,
                    0x03,
                    Zero,
                    0xFE
                }
            })
            Name (_HID, "PNP0A08" /* PCI Express Bus */)  // _HID: Hardware ID
            Name (_CID, "PNP0A03" /* PCI Bus */)  // _CID: Compatible ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Name (_BBN, Zero)  // _BBN: BIOS Bus Number
            Method (_UID, 0, NotSerialized)  // _UID: Unique ID
            {
                Return (_SEG) /* \_SB_.PCI1._SEG */
            }

            Method (_INI, 0, Serialized)  // _INI: Initialize
            {
                OperationRegion (OCFG, SystemMemory, (CFGB + 0x9000), 0x04)
                Field (OCFG, DWordAcc, NoLock, Preserve)
                {
                    CFGI,   32
                }

                CFGI = 0x00100000
            }

            Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
            {
                Name (RBUF, ResourceTemplate ()
                {
                    WordBusNumber (ResourceProducer, MinFixed, MaxFixed, PosDecode,
                        0x0000,             // Granularity
                        0x0000,             // Range Minimum
                        0x0000,             // Range Maximum
                        0x0000,             // Translation Offset
                        0x0001,             // Length
                        ,, _Y06)
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y07, AddressRangeMemory, TypeStatic)
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y08, AddressRangeMemory, TypeStatic)
                })
                CreateWordField (RBUF, \_SB.PCI1._CRS._Y06._MIN, MI00)  // _MIN: Minimum Base Address
                CreateWordField (RBUF, \_SB.PCI1._CRS._Y06._MAX, MA00)  // _MAX: Maximum Base Address
                CreateWordField (RBUF, \_SB.PCI1._CRS._Y06._TRA, TR00)  // _TRA: Translation
                CreateWordField (RBUF, \_SB.PCI1._CRS._Y06._LEN, LE00)  // _LEN: Length
                LE00 = ((PBMA - _BBN) + One)
                MI00 = _BBN /* \_SB_.PCI1._BBN */
                TR00 = Zero
                MA00 = ((MI00 + LE00) - One)
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y07._MIN, MI01)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y07._MAX, MA01)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y07._TRA, TR01)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y07._LEN, LE01)  // _LEN: Length
                LE01 = MS32 /* \_SB_.MS32 */
                MI01 = BB32 /* \_SB_.BB32 */
                TR01 = (MB32 - BB32) /* \_SB_.BB32 */
                MA01 = ((MI01 + LE01) - One)
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y08._MIN, MI02)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y08._MAX, MA02)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y08._TRA, TR02)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.PCI1._CRS._Y08._LEN, LE02)  // _LEN: Length
                LE02 = MS64 /* \_SB_.PCI1.MS64 */
                MI02 = MB64 /* \_SB_.PCI1.MB64 */
                TR02 = Zero
                MA02 = ((MI02 + LE02) - One)
                Return (RBUF) /* \_SB_.PCI1._CRS.RBUF */
            }

            Device (RES0)
            {
                Name (_HID, "AMZN0001")  // _HID: Hardware ID
                Name (_CID, "PNP0C02" /* PNP Motherboard Resources */)  // _CID: Compatible ID
                Method (_UID, 0, NotSerialized)  // _UID: Unique ID
                {
                    Return (_SEG) /* \_SB_.PCI1._SEG */
                }

                Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
                {
                    Name (RBUF, ResourceTemplate ()
                    {
                        QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                            0x0000000000000000, // Granularity
                            0x0000000000000000, // Range Minimum
                            0x0000000000000000, // Range Maximum
                            0x0000000000000000, // Translation Offset
                            0x0000000000000001, // Length
                            ,, _Y09, AddressRangeMemory, TypeStatic)
                    })
                    CreateQWordField (RBUF, \_SB.PCI1.RES0._CRS._Y09._MIN, MI00)  // _MIN: Minimum Base Address
                    CreateQWordField (RBUF, \_SB.PCI1.RES0._CRS._Y09._MAX, MA00)  // _MAX: Maximum Base Address
                    CreateQWordField (RBUF, \_SB.PCI1.RES0._CRS._Y09._TRA, TR00)  // _TRA: Translation
                    CreateQWordField (RBUF, \_SB.PCI1.RES0._CRS._Y09._LEN, LE00)  // _LEN: Length
                    LE00 = 0x1000
                    MI00 = (CFGB + 0x8000)
                    TR00 = Zero
                    MA00 = ((MI00 + LE00) - One)
                    Return (RBUF) /* \_SB_.PCI1.RES0._CRS.RBUF */
                }
            }

            Name (SUPP, Zero)
            Name (CTRL, Zero)
            Method (_OSC, 4, NotSerialized)  // _OSC: Operating System Capabilities
            {
                If ((Arg0 == ToUUID ("33db4d5b-1ff7-401c-9657-7441c03dd766") /* PCI Host Bridge Device */))
                {
                    CreateDWordField (Arg3, Zero, CDW1)
                    CreateDWordField (Arg3, 0x04, CDW2)
                    CreateDWordField (Arg3, 0x08, CDW3)
                    SUPP = CDW2 /* \_SB_.PCI1._OSC.CDW2 */
                    CTRL = CDW3 /* \_SB_.PCI1._OSC.CDW3 */
                    CTRL &= 0x1E
                    CTRL &= 0x1D
                    If ((Arg1 != One))
                    {
                        CDW1 |= 0x08
                    }

                    If ((CDW3 != CTRL))
                    {
                        CDW1 |= 0x10
                    }

                    CDW3 = CTRL /* \_SB_.PCI1.CTRL */
                    Return (Arg3)
                }
                Else
                {
                    CDW1 |= 0x04
                    Return (Arg3)
                }
            }
        }

        Device (PCI2)
        {
            Name (_SEG, 0x02)  // _SEG: PCI Segment
            Name (_STA, 0x00)  // _STA: Status
            Name (CFGB, 0x0000001000120000)
            Name (CFGS, 0x9310)
            Name (MB32, 0x0000001F00000000)
            Name (MB64, 0x0000001C00000000)
            Name (MS64, 0x0000000300000000)
            Name (_PRT, Package (0x04)  // _PRT: PCI Routing Table
            {
                Package (0x04)
                {
                    0xFFFF,
                    Zero,
                    Zero,
                    0x0105
                },

                Package (0x04)
                {
                    0xFFFF,
                    One,
                    Zero,
                    0x0106
                },

                Package (0x04)
                {
                    0xFFFF,
                    0x02,
                    Zero,
                    0x0107
                },

                Package (0x04)
                {
                    0xFFFF,
                    0x03,
                    Zero,
                    0x0108
                }
            })
            Name (_HID, "PNP0A08" /* PCI Express Bus */)  // _HID: Hardware ID
            Name (_CID, "PNP0A03" /* PCI Bus */)  // _CID: Compatible ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Name (_BBN, Zero)  // _BBN: BIOS Bus Number
            Method (_UID, 0, NotSerialized)  // _UID: Unique ID
            {
                Return (_SEG) /* \_SB_.PCI2._SEG */
            }

            Method (_INI, 0, Serialized)  // _INI: Initialize
            {
                OperationRegion (OCFG, SystemMemory, (CFGB + 0x9000), 0x04)
                Field (OCFG, DWordAcc, NoLock, Preserve)
                {
                    CFGI,   32
                }

                CFGI = 0x00100000
            }

            Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
            {
                Name (RBUF, ResourceTemplate ()
                {
                    WordBusNumber (ResourceProducer, MinFixed, MaxFixed, PosDecode,
                        0x0000,             // Granularity
                        0x0000,             // Range Minimum
                        0x0000,             // Range Maximum
                        0x0000,             // Translation Offset
                        0x0001,             // Length
                        ,, _Y0A)
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y0B, AddressRangeMemory, TypeStatic)
                    QWordMemory (ResourceProducer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y0C, AddressRangeMemory, TypeStatic)
                })
                CreateWordField (RBUF, \_SB.PCI2._CRS._Y0A._MIN, MI00)  // _MIN: Minimum Base Address
                CreateWordField (RBUF, \_SB.PCI2._CRS._Y0A._MAX, MA00)  // _MAX: Maximum Base Address
                CreateWordField (RBUF, \_SB.PCI2._CRS._Y0A._TRA, TR00)  // _TRA: Translation
                CreateWordField (RBUF, \_SB.PCI2._CRS._Y0A._LEN, LE00)  // _LEN: Length
                LE00 = ((PBMA - _BBN) + One)
                MI00 = _BBN /* \_SB_.PCI2._BBN */
                TR00 = Zero
                MA00 = ((MI00 + LE00) - One)
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0B._MIN, MI01)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0B._MAX, MA01)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0B._TRA, TR01)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0B._LEN, LE01)  // _LEN: Length
                LE01 = MS32 /* \_SB_.MS32 */
                MI01 = BB32 /* \_SB_.BB32 */
                TR01 = (MB32 - BB32) /* \_SB_.BB32 */
                MA01 = ((MI01 + LE01) - One)
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0C._MIN, MI02)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0C._MAX, MA02)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0C._TRA, TR02)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.PCI2._CRS._Y0C._LEN, LE02)  // _LEN: Length
                LE02 = MS64 /* \_SB_.PCI2.MS64 */
                MI02 = MB64 /* \_SB_.PCI2.MB64 */
                TR02 = Zero
                MA02 = ((MI02 + LE02) - One)
                Return (RBUF) /* \_SB_.PCI2._CRS.RBUF */
            }

            Device (RES0)
            {
                Name (_HID, "AMZN0001")  // _HID: Hardware ID
                Name (_CID, "PNP0C02" /* PNP Motherboard Resources */)  // _CID: Compatible ID
                Method (_UID, 0, NotSerialized)  // _UID: Unique ID
                {
                    Return (_SEG) /* \_SB_.PCI2._SEG */
                }

                Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
                {
                    Name (RBUF, ResourceTemplate ()
                    {
                        QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                            0x0000000000000000, // Granularity
                            0x0000000000000000, // Range Minimum
                            0x0000000000000000, // Range Maximum
                            0x0000000000000000, // Translation Offset
                            0x0000000000000001, // Length
                            ,, _Y0D, AddressRangeMemory, TypeStatic)
                    })
                    CreateQWordField (RBUF, \_SB.PCI2.RES0._CRS._Y0D._MIN, MI00)  // _MIN: Minimum Base Address
                    CreateQWordField (RBUF, \_SB.PCI2.RES0._CRS._Y0D._MAX, MA00)  // _MAX: Maximum Base Address
                    CreateQWordField (RBUF, \_SB.PCI2.RES0._CRS._Y0D._TRA, TR00)  // _TRA: Translation
                    CreateQWordField (RBUF, \_SB.PCI2.RES0._CRS._Y0D._LEN, LE00)  // _LEN: Length
                    LE00 = 0x1000
                    MI00 = (CFGB + 0x8000)
                    TR00 = Zero
                    MA00 = ((MI00 + LE00) - One)
                    Return (RBUF) /* \_SB_.PCI2.RES0._CRS.RBUF */
                }
            }

            Name (SUPP, Zero)
            Name (CTRL, Zero)
            Method (_OSC, 4, NotSerialized)  // _OSC: Operating System Capabilities
            {
                If ((Arg0 == ToUUID ("33db4d5b-1ff7-401c-9657-7441c03dd766") /* PCI Host Bridge Device */))
                {
                    CreateDWordField (Arg3, Zero, CDW1)
                    CreateDWordField (Arg3, 0x04, CDW2)
                    CreateDWordField (Arg3, 0x08, CDW3)
                    SUPP = CDW2 /* \_SB_.PCI2._OSC.CDW2 */
                    CTRL = CDW3 /* \_SB_.PCI2._OSC.CDW3 */
                    CTRL &= 0x1E
                    CTRL &= 0x1D
                    If ((Arg1 != One))
                    {
                        CDW1 |= 0x08
                    }

                    If ((CDW3 != CTRL))
                    {
                        CDW1 |= 0x10
                    }

                    CDW3 = CTRL /* \_SB_.PCI2.CTRL */
                    Return (Arg3)
                }
                Else
                {
                    CDW1 |= 0x04
                    Return (Arg3)
                }
            }
        }

        Device (RP1B)
        {
            Name (_HID, "ACPI0004" /* Module Device */)  // _HID: Hardware ID
            Name (_UID, One)  // _UID: Unique ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Name (PBAR, 0x0000001F00000000)
            Name (PINT, 0x0105)
            Method (_STA, 0, NotSerialized)  // _STA: Status
            {
                If ((PBAR == 0xABCDEF0123456789))
                {
                    Return (Zero)
                }

                Return (0x0F)
            }

            Device (XHC0)
            {
                Name (_HID, "PNP0D10" /* XHCI USB Controller with debug */)  // _HID: Hardware ID
                Name (_UID, Zero)  // _UID: Unique ID
                Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
                {
                    Name (RBUF, ResourceTemplate ()
                    {
                        QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                            0x0000000000000000, // Granularity
                            0x0000000000000000, // Range Minimum
                            0x0000000000000000, // Range Maximum
                            0x0000000000000000, // Translation Offset
                            0x0000000000000001, // Length
                            ,, _Y0E, AddressRangeMemory, TypeStatic)
                        Interrupt (ResourceConsumer, Level, ActiveHigh, Shared, ,, _Y0F)
                        {
                            0x00000000,
                        }
                    })
                    CreateQWordField (RBUF, \_SB.RP1B.XHC0._CRS._Y0E._MIN, MI00)  // _MIN: Minimum Base Address
                    CreateQWordField (RBUF, \_SB.RP1B.XHC0._CRS._Y0E._MAX, MA00)  // _MAX: Maximum Base Address
                    CreateQWordField (RBUF, \_SB.RP1B.XHC0._CRS._Y0E._LEN, LE00)  // _LEN: Length
                    LE00 = 0x00100000
                    MI00 = (PBAR + 0x00200000)
                    MA00 = ((MI00 + LE00) - One)
                    CreateDWordField (RBUF, \_SB.RP1B.XHC0._CRS._Y0F._INT, IN01)  // _INT: Interrupts
                    IN01 = PINT /* \_SB_.RP1B.PINT */
                    Return (RBUF) /* \_SB_.RP1B.XHC0._CRS.RBUF */
                }
            }

            Device (XHC1)
            {
                Name (_HID, "PNP0D10" /* XHCI USB Controller with debug */)  // _HID: Hardware ID
                Name (_UID, One)  // _UID: Unique ID
                Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
                {
                    Name (RBUF, ResourceTemplate ()
                    {
                        QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                            0x0000000000000000, // Granularity
                            0x0000000000000000, // Range Minimum
                            0x0000000000000000, // Range Maximum
                            0x0000000000000000, // Translation Offset
                            0x0000000000000001, // Length
                            ,, _Y10, AddressRangeMemory, TypeStatic)
                        Interrupt (ResourceConsumer, Level, ActiveHigh, Shared, ,, _Y11)
                        {
                            0x00000000,
                        }
                    })
                    CreateQWordField (RBUF, \_SB.RP1B.XHC1._CRS._Y10._MIN, MI00)  // _MIN: Minimum Base Address
                    CreateQWordField (RBUF, \_SB.RP1B.XHC1._CRS._Y10._MAX, MA00)  // _MAX: Maximum Base Address
                    CreateQWordField (RBUF, \_SB.RP1B.XHC1._CRS._Y10._LEN, LE00)  // _LEN: Length
                    LE00 = 0x00100000
                    MI00 = (PBAR + 0x00300000)
                    MA00 = ((MI00 + LE00) - One)
                    CreateDWordField (RBUF, \_SB.RP1B.XHC1._CRS._Y11._INT, IN01)  // _INT: Interrupts
                    IN01 = PINT /* \_SB_.RP1B.PINT */
                    Return (RBUF) /* \_SB_.RP1B.XHC1._CRS.RBUF */
                }
            }
        }

        Name (SDCM, Zero)
        Name (SDLU, One)
        Device (SDC0)
        {
            Method (_HID, 0, NotSerialized)  // _HID: Hardware ID
            {
                If ((SDCM == One))
                {
                    Return ("80860F16")
                }
                Else
                {
                    Return ("BRCM5D12")
                }
            }

            Name (_CID, Package (0x02)  // _CID: Compatible ID
            {
                "80860F16",
                "VEN_8086&DEV_0F14"
            })
            Name (_UID, Zero)  // _UID: Unique ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
            {
                Name (RBUF, ResourceTemplate ()
                {
                    QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y12, AddressRangeMemory, TypeStatic)
                    Interrupt (ResourceConsumer, Level, ActiveHigh, Exclusive, ,, )
                    {
                        0x00000131,
                    }
                })
                CreateQWordField (RBUF, \_SB.SDC0._CRS._Y12._MIN, MI00)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.SDC0._CRS._Y12._MAX, MA00)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.SDC0._CRS._Y12._TRA, TR00)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.SDC0._CRS._Y12._LEN, LE00)  // _LEN: Length
                LE00 = 0x0260
                MI00 = 0x0000001000FFF000
                TR00 = Zero
                MA00 = ((MI00 + LE00) - One)
                Return (RBUF) /* \_SB_.SDC0._CRS.RBUF */
            }

            OperationRegion (GPIO, SystemMemory, 0x000000107D517C00, 0x40)
            Field (GPIO, DWordAcc, NoLock, Preserve)
            {
                Offset (0x04),
                DATA,   32
            }

            Method (_INI, 0, Serialized)  // _INI: Initialize
            {
                DATA &= 0xFFFFFFFFFFFFFFF7
            }

            Method (_DSM, 4, Serialized)  // _DSM: Device-Specific Method
            {
                If ((Arg0 == ToUUID ("f6c13ea5-65cd-461f-ab7a-29f7e8d5bd61") /* Unknown UUID */))
                {
                    If ((Arg1 >= Zero))
                    {
                        Switch (ToInteger (Arg2))
                        {
                            Case (Zero)
                            {
                                Return (Buffer (0x02)
                                {
                                     0x19, 0x01                                       // ..
                                })
                            }
                            Case (0x03)
                            {
                                DATA |= 0x08
                                Return (Buffer (One)
                                {
                                     0x00                                             // .
                                })
                            }
                            Case (0x04)
                            {
                                DATA &= 0xFFFFFFFFFFFFFFF7
                                Return (Buffer (One)
                                {
                                     0x00                                             // .
                                })
                            }
                            Case (0x08)
                            {
                                If ((SDLU == One))
                                {
                                    Return (Buffer (One)
                                    {
                                         0x02                                             // .
                                    })
                                }
                                Else
                                {
                                    Return (Buffer (One)
                                    {
                                         0x0F                                             // .
                                    })
                                }
                            }

                        }
                    }
                }

                Return (Buffer (One)
                {
                     0x00                                             // .
                })
            }

            Method (_DSD, 0, Serialized)  // _DSD: Device-Specific Data
            {
                Name (CAPM, Zero)
                CAPM |= 0x0000C00000000000
                If ((SDLU == One))
                {
                    CAPM |= 0x0000000300000000
                    If ((SDCM != One))
                    {
                        CAPM |= 0x0000000400000000
                    }
                }

                Return (Package (0x02)
                {
                    ToUUID ("daffd814-6eba-4d8c-8a91-bc9bbf4aa301") /* Device Properties for _DSD */,
                    Package (0x01)
                    {
                        Package (0x02)
                        {
                            "sdhci-caps-mask",
                            CAPM
                        }
                    }
                })
            }

            Device (SDMM)
            {
                Name (_ADR, Zero)  // _ADR: Address
                Method (_RMV, 0, NotSerialized)  // _RMV: Removal Status
                {
                    Return (One)
                }
            }
        }

        Device (SDC1)
        {
            Name (_HID, "BRCM5D12")  // _HID: Hardware ID
            Name (_CID, Package (0x02)  // _CID: Compatible ID
            {
                "80860F16",
                "VEN_8086&DEV_0F14"
            })
            Name (_UID, One)  // _UID: Unique ID
            Name (_CCA, Zero)  // _CCA: Cache Coherency Attribute
            Method (_CRS, 0, Serialized)  // _CRS: Current Resource Settings
            {
                Name (RBUF, ResourceTemplate ()
                {
                    QWordMemory (ResourceConsumer, PosDecode, MinFixed, MaxFixed, NonCacheable, ReadWrite,
                        0x0000000000000000, // Granularity
                        0x0000000000000000, // Range Minimum
                        0x0000000000000000, // Range Maximum
                        0x0000000000000000, // Translation Offset
                        0x0000000000000001, // Length
                        ,, _Y13, AddressRangeMemory, TypeStatic)
                    Interrupt (ResourceConsumer, Level, ActiveHigh, Exclusive, ,, )
                    {
                        0x00000132,
                    }
                })
                CreateQWordField (RBUF, \_SB.SDC1._CRS._Y13._MIN, MI00)  // _MIN: Minimum Base Address
                CreateQWordField (RBUF, \_SB.SDC1._CRS._Y13._MAX, MA00)  // _MAX: Maximum Base Address
                CreateQWordField (RBUF, \_SB.SDC1._CRS._Y13._TRA, TR00)  // _TRA: Translation
                CreateQWordField (RBUF, \_SB.SDC1._CRS._Y13._LEN, LE00)  // _LEN: Length
                LE00 = 0x0260
                MI00 = 0x0000001001100000
                TR00 = Zero
                MA00 = ((MI00 + LE00) - One)
                Return (RBUF) /* \_SB_.SDC1._CRS.RBUF */
            }

            Method (_DSM, 4, Serialized)  // _DSM: Device-Specific Method
            {
                If ((Arg0 == ToUUID ("f6c13ea5-65cd-461f-ab7a-29f7e8d5bd61") /* Unknown UUID */))
                {
                    If ((Arg1 >= Zero))
                    {
                        Switch (ToInteger (Arg2))
                        {
                            Case (Zero)
                            {
                                Return (Buffer (0x02)
                                {
                                     0x01, 0x01                                       // ..
                                })
                            }
                            Case (0x08)
                            {
                                Return (Buffer (One)
                                {
                                     0x02                                             // .
                                })
                            }

                        }
                    }
                }

                Return (Buffer (One)
                {
                     0x00                                             // .
                })
            }

            Method (_DSD, 0, Serialized)  // _DSD: Device-Specific Data
            {
                Return (Package (0x02)
                {
                    ToUUID ("daffd814-6eba-4d8c-8a91-bc9bbf4aa301") /* Device Properties for _DSD */,
                    Package (0x01)
                    {
                        Package (0x02)
                        {
                            "sdhci-caps-mask",
                            0x0000C00300000000
                        }
                    }
                })
            }

            Device (WLAN)
            {
                Name (_ADR, One)  // _ADR: Address
                Method (_RMV, 0, NotSerialized)  // _RMV: Removal Status
                {
                    Return (Zero)
                }
            }
        }
    }
}
